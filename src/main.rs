// # Веб-сервис секретного Санты.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tide::{Request, Response};
use serde_json::{Value, json, Map};

#[derive(PartialEq,Eq, Clone)]
enum Access
{
    User,
    Admin,
}

type Id = u32;

#[derive(Eq, Hash, PartialEq, Clone, serde::Serialize)]
struct UserGroupId
{
    user_id: Id,
    group_id: Id,
}
#[derive(Clone)]
struct UserGroupProps
{
    access_level: Access,
    santa_id: Id,
}
impl UserGroupProps {
    fn new(access_level: Access) -> UserGroupProps {
        UserGroupProps {
            access_level,
            santa_id: 0,
        }
    }
}

struct DataBase
{
    users: HashMap<Id, String>,
    users_max_id: Id,
    groups: HashMap<Id, bool>,
    groups_max_id: Id,
    user_groups: HashMap<UserGroupId, UserGroupProps>,
}

fn get_field<T>(object: &serde_json::Map<String, Value>, key: &str) -> T
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    object.get(key).unwrap().as_str().unwrap().parse().unwrap()
}

fn response_data(value: Value) -> Response
{
    Response::builder(200)
        .body(tide::Body::from_json(&value).unwrap())
        .build()
}

fn response_empty() -> Response
{
    Response::builder(200).build()
}

fn response_error(msg: &str) -> Response
{
    Response::builder(400)
        .body(tide::Body::from_json(&json!({"error": msg})).unwrap())
        .build()
}




fn user_create(input_obj: &Map<String, Value>, state: &Arc<Mutex<DataBase>>) -> Response
{
    let name: String = get_field(input_obj, "name");
    if name.len() > 0
    {
        let mut guard = state.lock().unwrap();
        let id = guard.users_max_id;
        guard.users.insert(id, name);
        guard.users_max_id += 1;

        response_data(json!({"id": id}))
    }
    else
    {
        response_error("bad name")
    }
}

fn does_user_belong_to_group(user_id: Id, group_id: Id, user_groups: &HashMap<UserGroupId,UserGroupProps>) -> bool
{
    return user_groups.contains_key(&UserGroupId { user_id, group_id });
}

fn count_admins(group_id: Id, user_groups: &HashMap<UserGroupId, UserGroupProps>) ->usize
{
    let iter = user_groups.into_iter();
    let collection = iter.filter(|&x| x.0.group_id == group_id && x.1.access_level == Access::Admin);
    return collection.count();
}
fn is_admin(user_id: Id, group_id: Id, map: &HashMap<UserGroupId, UserGroupProps>) -> bool
{
    map.get(
        &UserGroupId {
            user_id,
            group_id,
        }
    ).unwrap().access_level == Access::Admin
}

fn main() -> Result<(), std::io::Error> 
{
    let f = async {
        let data = DataBase
        {
            users: HashMap::new(),
            users_max_id: 0,
            groups: HashMap::new(),
            groups_max_id: 0,
            user_groups: HashMap::new(),
        };
        let state = Arc::new(Mutex::new(data));
        let mut app = tide::with_state(state);

        // Routes
        app.at("/users")
            .get(|request: Request<Arc<Mutex<DataBase>>>| async move {
                let guard = request.state().lock().unwrap();
                Ok(json!(guard.users))
            });
        app.at("/groups")
            .get(|request: Request<Arc<Mutex<DataBase>>>| async move {
                let guard = request.state().lock().unwrap();
                Ok(json!(guard.groups))
            });
        
        app.at("/user/create")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let input_obj = body.as_object().unwrap();
                Ok(user_create(input_obj, request.state()))
            });
        app.at("/group/create")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let creator_id: Id = get_field(object, "creator_id");

                let mut guard = request.state().lock().unwrap();
                Ok(if !guard.users.contains_key(&creator_id)
                {
                    response_error("no such user")
                }
                else
                {
                    let id = guard.groups_max_id;
                    guard.groups.insert(id, false);
                    guard.groups_max_id += 1;
                    guard.user_groups.insert(
                        UserGroupId
                        {
                            user_id: creator_id,
                            group_id: id,
                        },
                        UserGroupProps
                        {
                            access_level: Access::Admin,
                            santa_id: 0,
                        }
                    );
                    response_data(json!({"group_id": id}))
                })
            });
        app.at("/group/join")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let value: Value = request.body_json().await.unwrap();
                let object = value.as_object().unwrap();
                let mut user_group_id = UserGroupId{user_id: 0, group_id: 0};
                user_group_id.user_id = get_field(object, "user_id");
                user_group_id.group_id = get_field(object, "group_id");

                let mut guard = request.state().lock().unwrap();
                Ok(match guard.groups.get(&user_group_id.group_id)
                {
                    None => response_error("no such group"),
                    Some(is_closed) =>
                    {
                        if *is_closed
                        {
                            response_error("group is closed")
                        }
                        else
                        {
                            if !guard.users.contains_key(&user_group_id.user_id)
                            {
                                response_error("no such user")
                            }
                            else
                            {
                                if guard.user_groups.contains_key(&user_group_id)
                                {
                                    response_error("user already in group")
                                }
                                else
                                {
                                    guard.user_groups.insert(user_group_id, UserGroupProps{access_level: Access::User, santa_id: 0});
                                    response_empty()
                                }
                            }
                        }
                    },
                })
            });
        app.at("/group/unadmin")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let admin_id = get_field(object, "admin_id");
                let group_id = get_field(object, "group_id");

                let mut guard = request.state().lock().unwrap();
                Ok(if !does_user_belong_to_group(admin_id, group_id, &guard.user_groups)
                {
                    response_error("User does not belong to this group. Try again.")
                }
                else 
                {
                    let ugid = UserGroupId { user_id: admin_id, group_id: group_id};
                    let ugp = guard.user_groups.get(&ugid).unwrap();
                    if ugp.access_level != Access::Admin
                    {
                        response_error("This user is not an admin.")
                    }
                    else
                    {
                        if count_admins(group_id, &guard.user_groups) < 2
                        {
                            response_error("It is impossible to remove the last admin in a group. You can appoint a new admin and repeat or delete the whole group.")
                        }
                        else
                        {
                            let mut ugp1 = guard.user_groups.get_mut(&ugid).unwrap();
                            ugp1.access_level = Access::User;
                            response_empty()
                        }
                    }
                })
            });
        app.at("/group/delete")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let admin_id = get_field(object, "admin_id");
                let group_id = get_field(object, "group_id");

                let mut guard = request.state().lock().unwrap();
                Ok(if !does_user_belong_to_group(admin_id, group_id, &guard.user_groups)
                {
                    response_error("User does not belong to this group. Try again.")
                }
                else
                {
                    let ugid = UserGroupId { user_id: admin_id, group_id: group_id};
                    let ugp = guard.user_groups.get(&ugid).unwrap();
                    if ugp.access_level != Access::Admin
                    {
                        response_error("This user is not an admin.")
                    }
                    else
                    {
                        // Before delete group, we need to delete all users from this group
                        guard.user_groups.retain(|user_group_id, _| {
                            user_group_id.group_id != group_id
                        });
                        guard.groups.remove(&group_id);
                        response_empty()
                    }
                }
            )});
        app.at("/group/make_admin")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let group_id: Id = get_field(object, "group_id");
                let member_id: Id = get_field(object, "member_id");
                let admin_id: Id = get_field(object, "admin_id");

                let mut guard = request.state().lock().unwrap();
                Ok(if !guard.groups.contains_key(&group_id)
                {
                    response_error("no such group")
                }
                else if !does_user_belong_to_group(member_id, group_id, &guard.user_groups)
                {
                    response_error("user isn't a member of the group")
                }
                else if is_admin(member_id, group_id, &guard.user_groups)
                {
                    response_error("user is already an admin")
                }
                else if !is_admin(admin_id, group_id, &guard.user_groups)
                {
                    response_error("admin_id isn't an actual admin's ID")
                }
                else {
                    guard.user_groups.insert(
                        UserGroupId {
                            user_id: member_id,
                            group_id,
                        },
                        UserGroupProps::new(Access::Admin),
                    );
                    response_empty()
                }
            )});
        app.at("/group/quit")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let group_id : Id = get_field(object, "group_id");
                let user_id : Id = get_field(object, "user_id");
                let guard = request.state().lock().unwrap();
                Ok(if !guard.user_groups.contains_key(&UserGroupId{ user_id: (user_id), group_id: (group_id) })
                {
                    response_error("bad group_id and/or user_id")
                }
                else
                {
                    let temp_user_group_id = guard.user_groups.get(&UserGroupId { user_id: (user_id), group_id: (group_id) });
                    if temp_user_group_id.unwrap().access_level == Access::Admin && count_admins(group_id, &guard.user_groups) < 2
                    {
                        response_error("user_id is only one Admin in group_id")
                    }
                    else
                    {   
                        response_empty()
                    }
                })
            });
            app.at("/group/target_by_id/:user_id/:group_id")
            .get(|mut request: Request<Arc<Mutex<DataBase>>>| async move{
                let first_id = request.param("user_id")?;
                let second_id = request.param("group_id")?;
                for c in first_id.chars() {
                    if !c.is_numeric() {
                        return Ok(response_error("Wrong format user id"));
                    } 
                }
                for c in second_id.chars() {
                    if !c.is_numeric() {
                        return Ok(response_error("Wrong format group id"));
                    } 
                }
                let user_id: Id = first_id.parse().unwrap();
                let group_id: Id = second_id.parse().unwrap();
                let from_user = UserGroupId{
                    user_id,
                    group_id
                };
                let mut guard = request.state().lock().unwrap();
                if guard.user_groups.contains_key(&from_user){
                    let need_user  = guard.user_groups.get(&from_user).unwrap();
                    let need_user_id = need_user.santa_id;
                    return Ok(response_data(json!({"cysh_for_id": need_user_id})));
                } else {
                    return Ok(response_error("bad user or group id"));
                }
            });

        app.at("/group/secret_santa")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let group_id: Id = get_field(object, "group_id");
                let admin_id: Id = get_field(object, "admin_id");
                let mut guard = request.state().lock().unwrap();
                let admin: &UserGroupProps;

                Ok(match guard.user_groups.get(&UserGroupId{user_id: admin_id, group_id: group_id}){
                    None => response_error("no such user"),
                    Some(res) => {
                        admin = res;
                        if admin.access_level == Access::Admin{
                            *guard.groups.get_mut(&(group_id)).unwrap() = true;


                            let mut count = 0;
                            let mut santas = Vec::new();

                            //Пользователю присваивается тайный кыш бабай с Id на 1 больше, чем у него.
                            //Если при итерации пользователь не из этой группы, то его тайный кыш бабай - тот пользователь,
                            //который был первым из тех пользователей этой группы, что шли друг за другом в общем списке.
                            //Например: из нужной группы пользователи с id 0, 1, 2, 4, 5. Тайный кыш бабай распределится таким образом
                            //0-1
                            //1-2
                            //2-0 (2+1=3) 3 нет в этой группе, значит вычитаем count = 3, получаем 0. И обнуляем его сразу
                            //4-5
                            //5-4 (5+1=6) 6 нет в этой группе, вычитаем count = 2, получаем 4
                            

                            for (key, mut val) in guard.user_groups.clone() {
                                if key.group_id == group_id {
                                    count += 1;
                                    let mut santa_id = key.user_id + 1;
                                    if !guard.user_groups.contains_key(&UserGroupId{user_id: santa_id, group_id: group_id})
                                    {
                                        santa_id -= count;
                                        count = 0;
                                    }
                                    santas.push((key.clone(), santa_id));
                                    val.santa_id = santa_id;

                                    *guard.user_groups.get_mut(&key).unwrap() = val;
                                }
                            }
                            response_data(json!(santas))

                        }
                        else {
                            response_error("something went wrong")
                        }
                    }
                })
            });

        app.listen("127.0.0.1:8080").await
    };
    futures::executor::block_on(f)
}