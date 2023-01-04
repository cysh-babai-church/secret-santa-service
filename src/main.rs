// # Веб-сервис секретного Санты.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tide::Request;
use serde_json::{Value, json};

#[derive(PartialEq)]
enum Access
{
    User,
    Admin,
}

type Id = u32;

#[derive(Eq, Hash, PartialEq)]
struct UserGroupId
{
    user_id: Id,
    group_id: Id,
}
struct UserGroupProps
{
    access_level: Access,
    santa_id: Id,
}

struct DataBase
{
    users: HashMap<Id, String>,
    groups: HashMap<Id, bool>,
    user_groups: HashMap<UserGroupId, UserGroupProps>,
}

fn get_not_used_in_map_id<T>(map: &HashMap<Id, T>) -> Id
{
    *map.keys().max().unwrap() + 1
}

fn main() -> Result<(), std::io::Error> 
{
    let f = async {
        let mut data = DataBase
        {
            users: HashMap::new(),
            groups: HashMap::new(),
            user_groups: HashMap::new(),
        };
        
        // Mock data (данные для тестирования)
        data.users.insert(0, "Ilya".to_string());
        data.users.insert(2, "Stepan".to_string());
        data.groups.insert(0, false);
        data.groups.insert(1, false);
        data.user_groups.insert(
            UserGroupId
            {
                user_id: 0,
                group_id: 0,
            },
            UserGroupProps
            {
                access_level: Access::Admin,
                santa_id: 0,
            }
        );
        data.user_groups.insert(
            UserGroupId
            {
                user_id: 2,
                group_id: 1,
            },
            UserGroupProps
            {
                access_level: Access::Admin,
                santa_id: 0,
            }
        );
     
        let state = Arc::new(Mutex::new(data));
        let mut app = tide::with_state(state);

        // Routes
        app.at("/users")
            .get(|request: Request<Arc<Mutex<DataBase>>>| async move {
                let state = request.state();
                let guard = state.lock().unwrap();
                Ok(json!(guard.users))
            });
        app.at("/groups")
            .get(|request: Request<Arc<Mutex<DataBase>>>| async move {
                let state = request.state();
                let guard = state.lock().unwrap();
                Ok(json!(guard.groups))
            });
        app.at("/user/create")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Option<Value> = request.body_json().await.ok();
                match body.and_then(
                    |value| value.as_object().and_then(
                    |object| object.get("name").and_then(
                    |value| value.as_str().and_then(
                    |name|
                    {
                        let state = request.state();
                        let mut guard = state.lock().unwrap();
                        let id = get_not_used_in_map_id(&guard.users);
                        guard.users.insert(id, name.to_string());
                        Some(json!({"id": id}))
                    }
                ))))
                {
                    Some(res) => Ok(res),
                    None => Ok(json!({"error": "cant read name"})),
                }
            });

        app.at("/group/secret_santa")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let groupId: Id = data.user_groups.get_field(object, "group_id");
                let adminId: Id = data.user_groups.get_field(object, "admin_id");

                let admin = match data.user_groups.get(&UserGroupId{user_id: adminId, group_id: groupId})
                {
                    Some(res) => res,
                    None => panic!()
                };
                if admin.access_level == Access::Admin{
                    data.groups.get_mut(&(groupId)) = false;
                    let mut count = 0;
                    let mut out = String::new();
                    out += "{\n";
                    for (key, val) in data.user_groups{
                        count += 1;
                        if key.group_id == groupId{
                            let santaId = key.user_id+1;
                            match data.user_groups.get(&UserGroupId{user_id: santaId, group_id: groupId})
                            {
                                Some(res) => {val.santa_id = santaId},
                                None => {
                                    santaId -= count;
                                    val.santa_id = santaId;
                                }
                            }
                            out += "\t\"";
                            out.push_str(&*(key.user_id.to_string()));
                            out += "\":\"";
                            out.push_str(&*(val.santa_id.to_string()));
                            out += "\"\n";
                        }
                    }
                    out += "}";
                    Ok(tide::Response::builder(200)
                        .body(tide::Body::from_json(&json!(out))?)
                        .build())
                }
                else {
                    Ok(tide::Response::builder(200)
                        .body(tide::Body::from_json(&json!({"error": "not an admin"}))?)
                        .build())
                }
            });
        app.listen("127.0.0.1:8080").await
    };
    
    futures::executor::block_on(f)
}
