# Методы

## GET /users - получить список пользователей

- Возвращает JSON объект соответствия между ID пользователя и его именем для всех пользователей сервиса.

```json
// Out
{
  "0": "Ilya",
  "1": "Stepan"
}
```

## GET /groups - получить список групп

- Возвращает JSON объект соответствия между ID группы и ее статусом закрытости для всех групп сервиса.

```json
// Out
{
  "0": true,
  "1": false
}
```

## POST /user/create - создать пользователя

- Принимает JSON объект с полем `name` равным требуемому имени нового пользователя. Возвращает JSON объект с полем `id` равным ID нового пользователя в случае успеха, код возврата `200`.
- Если имя - пустая строка, возвращает JSON объект с полем `error` равным сообщению об ошибке, код возврата `400`.
- Если входные данные - не JSON объект с полем `name`, содержащим строку, то сервер паникует, а клиент не получает ответ на свой запрос.

Пример правильного обмена данными:

```json
// In
{
  "name":"Danis"
}

// Out
{
  "id":2
}
```

## POST /group/create - создать группу

- Принимает JSON объект с полем `creator_id` - ID создателя группы. Возвращает JSON объект с полем `id` равным ID новой группы в случае успеха, код возврата `200`.
- Если пользователя `creator_id` нет, возвращает JSON объект с полем `error` равным сообщению об ошибке, код возврата `400`.
- Если входные данные - не JSON объект с полем `creator_id`, содержащим число в форме строки, то сервер паникует, а клиент не получает ответ на свой запрос.

Пример нормального обмена данными

```json
// In
{
  "creator_id":"3"
}

// Out
{
  "group_id":4
}
```

## DELETE /group/delete

- Удаляет группу по `group_id` и `admin_id`. Если пользователь с `admin_id` не является администратором этой группы, выдает код `403` с сообщением `User does not belong to this group. Try again.` или `This user is not an admin.`.
- Также удаляет из группы всех участников.

```json
// In
{
  "admin_id":2,
  "group_id":4,
}

// Out
{}
```

## POST /group/join

Предназначен для добавления пользователя с `user_id` в группу с `group_id` в качестве обычного пользователя.

- Принимает JSON объект с полями `user_id` и `group_id`.

Назовем ошибкой http-ответ с кодом `400` и телом в виде JSON объекта с полем `error` равным строке, которую назовем сообщением ошибки.

- Если нет чисел `user_id` и `group_id`, не отвечает.
- Иначе если нет группы с `group_id`, возвращает ошибку с сообщением `"no such group"`.
- Иначе если она закрыта, возвращает ошибку с сообщением `"group is closed"`.
- Иначе если пользователя с `user_id` нет, возвращает ошибку с сообщением `"no such user"`.
- Иначе если пользователь с `user_id` уже в этой группе, возвращает ошибку с сообщением `"user already in group"`.
- Иначе добавляет пользователя в группу и возвращает ответ с кодом `200` и пустым телом.

Пример входных данных

```json
{
  "user_id": 2,
  "group_id": 0
}
```

## POST /group/make_admin - дать пользователю права администратора
- Принимает JSON-объект с полями:
  - `member_id` - ID пользователя.
  - `group_id` - ID группы.
  - `admin_id` - ID уже имеющегося администратора.
- Делает пользователя с `member_id` администратором группы - `group_id`, если `admin_id` это ID администратора, `member_id` уже является участником этой группы и `admin_id` не равен `member_id`.
- В случае успеха возвращает пустой JSON-объект, код возврата `200`.
- Если `group_id` отсутствует в базе данных, возвращает ошибку с сообщением `"no such group"`.
- Если `member_id` не является участником группы, возвращает ошибку с сообщением `"user isn't a member of the group"`.
- Если `member_id` уже является администратором, возвращает ошибку с сообщением `"user is already an admin"`.
- Если `admin_id` не является администратором, возвращает ошибку с сообщением `"admin_id isn't an actual admin's ID"`.

Пример входных данных:
```json
{
  "member_id":"3",
  "group_id":"1",
  "admin_id":"2",
}
```

## GET /group/list_admins/:group_id - получить список администраторов группы
- Для группы `group_id` возвращает список её администраторов.
- Принимает в URL запроса `group_id` нужной группы.
- В случае успеха возвращает JSON-объект, содержащий ID администраторов в качестве полей и их имена в качестве значений, код возврата `200`.
- Если ID группы введён некорректно, возвращает ошибку с сообщением `"wrong format group_id"`.
- Если группы с введённым ID нет в базе данных, возвращает ошибку с сообщением `"no such group"`.

Пример нормального обмена данными:
```url
http://127.0.0.1:8080/group/list_admins/0
```
```json
// Out
{
  "0":"Admin name",
  "1":"Another admin",
}
```

## POST /group/quit - исключить из группы
- Принимает JSON объект с полями
  - `group_id` равным ID группы.
  - `user_id` равным ID пользователя.
- Исключает пользователя с `user_id` из группы `group_id`, если `user_id` не единственный администратор этой группы.
- В случае успеха возвращает код возврата `200`.
- В случае отсутствия пользователя и/или группы возвращает JSON объект с полем `error`, равным сообщению об ошибке, код возврата `400`.
- Если пользователь `user_id` единственный администратор группы, возвращает JSON объект с полем `error`, равным сообщению об ошибке, код возврата `401`. // OH

Пример правильного обмена данных:
```json
// In
{
  "group_id":"4",
  "user_id":"2",
}

// Out
{}
```

## POST /group/unadmin

Делает пользователя с `admin_id` пользователем группы `group_id`, если `admin_id` это `id` администратора.

- Принимает JSON объект с полями `admin_id` и `group_id`.

Назовем ошибкой http-ответ с кодом 400 и телом в виде JSON объекта с полем error равным строке, которую назовем сообщением ошибки.

- Если нет чисел `admin_id` или `group_id`, не отвечает.
- Если пользователя нет в группе, или группа указана не та, ошибка с сообщением: `"User does not belong to this group. Try again."`
- Если пользователь принадлежит группе, но не является её администратором, ошибка с сообщением: `"This user is not an admin."`
- Если указанный id принадлежит последнему администратору группы, ошибка с сообщением: `"It is impossible to remove the last admin in a group. You can appoint a new admin and repeat or delete the whole group."`
- Если ни один из этих пунктов не выполняется - `access_level` меняется до `user`.

Пример входных данных
```json
{
  "admin_id": "3",
  "group_id": "4"
}
```

## GET /group/target_by_id

- Для пользователя `user_id` в группе `group_id`, возвращает `cysh_for_id` того пользователя, для кого `user_id` стал тайным Кыш Бабаем.
- Принимает в URL запроса `user_id` нужного пользователя и `group_id` нужной группы. 
- Возвращает JSON объект с полем `cysh_for_id` с нужным ID в случае успеха, код возврата `200`.
- Если введены некорректные данные (например вместо числа ввели символы) - возвращает JSON объект с полем `error` равным сообщению об ошибке, код возврата `400`.
- Если нет такого пользователя, или нет такой группы, или нет пользователя в группе, или пользователю еще не назначен Кыш Бабай, возвращает JSON объект с полем error равным сообщению об ошибке, код возврата `400`.

Пример:
```url
http://127.0.0.1:8080/group/target_by_id/{user_id}/{group_id}
-------------------
http://127.0.0.1:8080/group/target_by_id/1/0

// Out
{
  "cysh_for_id":0
}
-------------------
http://127.0.0.1:8080/group/target_by_id/bc/0

// Out
{
  "error": "Wrong format user id"
}
-------------------
http://127.0.0.1:8080/group/target_by_id/0/ab

// Out
{
  "error": "Wrong format group id"
}
```

## POST /group/secret_santa

- Запускает Тайного Кыш Бабая в группе `group_id`, если `admin_id` это id администратора группы `group_id`.
- Тайный Кыш Бабай:
  1. Закрыть группу
  2. Выставить всем участникам группы `group_id` того пользователя, для кого они стали тайным Кыш Бабаем.

```json
{
  "group_id":4,
  "admin_id":3,
}

// Out
{}
```

## PUT /user/update

- Принимает JSON-объект с полями:
  - `user_id` - ID пользователя.
  - `name` - новое имя пользователя.
- Изменяет имя пользователя с `user_id` на имя `name`.
- В случае успеха возвращает пустой JSON-объект, код возврата `200`.
- Если `user_id` отсутствует в базе данных, возвращает ошибку с сообщением `"no such id"`.

```json
// In
{
  "user_id":2,
  "name":"Новое Имя"
}
// Out
{}
```

## DELETE /user/delete

Удаление пользователя с `user_id`.

+ Eсли такой пользователь не найден, результат: ошибка с сообщением: `"This user does not exist."`
+ Иначе
  + Если `user_gropus` пустое, то происходит удаление из `users`
  + Иначе
    + Если у пользователя нет закрытых групп, он удаляется из всех открытых, кроме тех, где он является последним администратором. 
      + Если нет групп, где он является администратором, то происходит всех полей из `user_groups` с его `user_id` и удаление из `users`
      + Если такие группы есть, результатом будет удаление из тех групп, где он не является администратором и ошибка с сообщением: `"User cannot be deleted from groups:..., because he is the last admin in these groups."`, где `...` - `group_id`, из которых удалить нельзя.
    + Если есть закрытые группы, аналогичное удаление из всех открытых групп, где пользователь не является единственный администратором, но не удаление из `users`
      + Например, если есть закрытые группы, но нет открытых, где пользователь - админ, удаление всех октрытых и ошибка: `"User has closed groups. So he was deleted from opened groups."`
      + Если есть закрытые и есть открытые, где он единственный админ, ошибка: `"User has closed groups. So he was deleted from opened groups, if he wasn't last admin. User cannot be delete from groups:..., because of last admin."`

Пример ввода:
```json
{
  "user_id":"2"
}
```

