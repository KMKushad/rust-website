/*
Explanation of all of the template files
----------------------------------------

base.html.tera
The base template, all other templates extend off of it. 
Contains all of the links to the other pages.

hello.html.tera
Home page, has and displays a given title.

login.html.tera
Has a form that submits a username and password to /login via a post request.

message_list.html.tera
Displays the titles of every message as a link and allows you to make a new message.

message.html.tera
Displays one message and the replies to it, and allows you to make a reply.

profile.html.tera
Displays the username and user id.

register.html.tera
Submits a username and password to /register via a post request to be added to the database.
*/

#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket::response::Redirect;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket_dyn_templates::{Template, context};
use either::*;
use serde::{Deserialize, Serialize};
use rusqlite::{Connection};

//A struct to hold account credentials from the login and registration forms.
#[derive(Debug, FromForm)]
struct AccountInfo {
    username: String,
    password: String,
}

#[derive(Debug, FromForm)]
struct DirectMessage {
    receiver: String,
    content: String,
}

//A struct to represent a user object, same as the database columns.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    password: String,
    time_created: String,
}

//A struct to represent a message, same as the database columns.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct Message {
    id: i32,
    title: String,
    content: String,
    username: String,
    time_created: String,
}

/*
A struct to represent a reply, same as the database columns.

The parent field represents the main Message database id that the reply is attached to.
Ex. A reply to Message(id: 3) would have the parent set to 3*/
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct Reply {
    parent: i32,
    id: i32,
    content: String,
    username: String,
    time_created: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct Conversation {
    id: i32,
    sender: String,
    receiver: String, 
    content: String,
    reference: Option<i32>,
    conversation_id: i32,
    time_created: String,
}

//Tuple struct to represent an ID.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct ID(i32);

/*
When a get request is made to /profile, this first checks if the user is logged in.

If so, they proceed to their user profile, where data such as username and id are displayed
*/
#[get("/profile")]
fn profile(cookies: &CookieJar<'_>) -> Either<Redirect, Template> {
    if logged_in(cookies) {
        let id = cookies.get("user_id");
        let name = cookies.get("username");

        Right (
            Template::render("profile", context! {
                username: name.map(|c| c.value()),
                password: id.map(|c| c.value()),
            })
        )
    }

    else {
        Left(Redirect::to("login"))
    }
}   

/*
Generic home page.
*/
#[get("/")]
fn index() -> Template {
    Template::render("hello", context! {
        title: String::from("Hello")
    })
}

/*
Once account details are submitted to the form, they are sent to /register via a post request.

This first checks for any duplicate usernames, and if there is not a duplicate the function 
adds the account to the database. 

It then sets the username and user_id cookies to the new account's credentials, and redirects to profile.
*/
#[post("/register", data = "<credentials>")]
fn create_new_account(cookies: &CookieJar<'_>, credentials: Form<AccountInfo>) -> Option<Either<Redirect, Template>> {
    println!("{:#?}", &credentials);

    let conn = Connection::open("forum.sqlite").ok()?;

    let mut stmt = conn.prepare("SELECT * FROM users WHERE username = ?1").ok()?;
    let rows = stmt.query_map([&credentials.username], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password: row.get(2)?,
            time_created: row.get(3)?,
        })
    }).ok()?;
    
    let mut users: Vec<User> = Vec::new();

    for user in rows {
            users.push(user.ok()?);
    }

    if !(users.len() == 0) {
        return Some(Right(Template::render("hello", context! {
            title: "Username already taken"
        })))
    }

    conn.execute(
        "INSERT INTO users (username, password) values (?1, ?2)",
        [&credentials.username, &credentials.password]
    ).ok()?;

    let mut stmt = conn.prepare("SELECT MAX(id) FROM users").ok()?;

    let rows = stmt.query_map([], |row| {
        Ok(ID(row.get(0)?))
    }).ok()?;

    let mut ids: Vec<ID> = Vec::new();

    for x in rows {
        ids.push(x.ok()?);
    }

    cookies.add(Cookie::new("user_id", ids[0].0.to_string()));
    cookies.add(Cookie::new("username", credentials.username.clone()));

    Some(Left(Redirect::to("profile")))
}   

/*
Allows a user to register an account only if they are not currently logged in.
*/
#[get("/register")]
fn render_register(cookies: &CookieJar<'_>) -> Either<Template, Redirect> {
    if !logged_in(cookies) {
        Left(Template::render("register", context! {}))
    }

    else {
        Right(Redirect::to("profile"))
    }
}

/*
Allows a user to log in to an account only if they are not currently logged in.
*/
#[get("/login")]
fn render_login(cookies: &CookieJar<'_>) -> Either<Template, Redirect> {
    if !logged_in(cookies) {
        Left(Template::render("login", context! {}))
    }

    else {
        Right(Redirect::to("profile"))
    }
}

/*
Once data is submitted, it is checked to see if it's a valid account.

If so, the user is logged in and the username and user_id cookies are set.
*/
#[post("/login", data = "<credentials>")]
fn login(cookies: &CookieJar<'_>, credentials: Form<AccountInfo>) -> Option<Either<Redirect, Template>> {
    let conn = Connection::open("forum.sqlite").ok()?;

    let mut stmt = conn.prepare("SELECT * FROM users WHERE username = ?1 AND password = ?2").ok()?;
    let rows = stmt.query_map([&credentials.username, &credentials.password], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password: row.get(2)?,
            time_created: row.get(3)?,
        })
    }).ok()?;

    let mut users: Vec<User> = Vec::new();

    for user in rows {
            users.push(user.ok()?);
    }

    if !(users.len() == 1) {
        return Some(Right(Template::render("hello", context! {
            title: "Invalid account"
        })))
    }

    else {
        cookies.add(Cookie::new("user_id", users[0].id.to_string()));
        cookies.add(Cookie::new("username", users[0].username.clone()));
    }

    Some(Left(Redirect::to("profile")))
}


/*
Deletes the username and user_id cookies, logging out the user.
*/
#[get("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Template {
    cookies.remove(Cookie::named("user_id"));
    cookies.remove(Cookie::named("username"));

    Template::render("hello", context!{
        title: "Successfully logged out."
    })
}

/* 
When the content of a message is inputted, an entry is submitted containing the following columns:
content, title, username.

Content is the submitted message, title is a placeholder for now (3/20/23), and the username is the logged in user.

It then redirects to the new message.
*/
#[post("/message", data = "<message>")]
fn submit(cookies: &CookieJar<'_>, message: Form<&str>) -> Option<Redirect> {
    let name = cookies.get("username");
    let conn = Connection::open("forum.sqlite").ok()?;

    println!("{}", message.to_string());

    conn.execute(
        "INSERT INTO messages (content, title, username) values (?1, ?2, ?3)",
        [message.to_string(), "test title".to_string(), name?.value().to_string()]
    ).ok()?;

    println!("Message sent");

    Some(Redirect::to("message"))
}

/*
This function displays the message list only if the user is logged in. 
*/
#[get("/message")]
fn messages(cookies: &CookieJar<'_>) -> Option<Template> {
    if logged_in(cookies) {
        let conn = Connection::open("forum.sqlite").ok()?;

        let mut stmt = conn.prepare("SELECT * FROM messages").ok()?;
        let rows = stmt.query_map([], |row| {
            Ok(Message {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                username: row.get(3)?,
                time_created: row.get(4)?,
            })
        }).ok()?;
    
        let mut messages: Vec<Message> = Vec::new();
    
        for message in rows {
            messages.push(message.ok()?);
        }
    
        Some(Template::render("message_list", context! {
            messages: messages,
        }))
    }

    else {
        Some(Template::render("login", context! {}))
    }
}

/*
The url format for messages is /message/<message_id>, so this method displays the message with the given id.

It also gets all of the replies to that message from the database and displays them too.
*/
#[get("/<message_id>")]
fn render_message(message_id: i32) -> Option<Template>{
    //code to find the message
    let conn = Connection::open("forum.sqlite").ok()?;

    let mut stmt = conn.prepare("SELECT * FROM messages WHERE id = ?1").ok()?;
    let message_rows = stmt.query_map([message_id], |row| {
        Ok(Message {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            username: row.get(3)?,
            time_created: row.get(4)?,
        })
    }).ok()?;

    let mut messages: Vec<Message> = Vec::new();

    for message in message_rows {
        messages.push(message.ok()?);
    }

    //code to find all of the replies
    stmt = conn.prepare("SELECT * FROM replies WHERE parent = ?1").ok()?;
    let reply_rows = stmt.query_map([message_id], |row| {
        Ok(Reply {
            id: row.get(0)?,
            parent: row.get(1)?,
            content: row.get(2)?,
            username: row.get(3)?,
            time_created: row.get(4)?,
        })
    }).ok()?;

    let mut replies: Vec<Reply> = Vec::new();

    for reply in reply_rows {
        replies.push(reply.ok()?);
    }

    Some(Template::render("message", context! {
        message: messages[0].clone(),
        replies: replies,
    }))
}

/*
This takes the input from a form in every rendered message and creates a reply.

The reply has a parent id, content, and the user who made it.

The parent id is the message that it's replying to, the content is the submitted message, and the username is the logged in user.
*/
#[post("/<message_id>/reply", data = "<content>", rank = 2)]
fn reply(cookies: &CookieJar<'_>, message_id: i32, content: Form<&str>) -> Option<Redirect>{
    let conn = Connection::open("forum.sqlite").ok()?;
    let username = cookies.get("username");

    conn.execute(
        "INSERT INTO replies (parent, content, username) values (?1, ?2, ?3)",
        [message_id.to_string(), content.to_string(), username?.value().to_string()]
    ).ok()?;

    Some(Redirect::to(format!("/message/{}", message_id.to_string())))
}


#[get("/conversations/<target>")]
fn render_conversation(cookies: &CookieJar<'_>, target: String) -> Option<Template>{
    if logged_in(cookies) {
        let name = cookies.get("username")?.value().to_string();
        let conn = Connection::open("forum.sqlite").ok()?;

        let mut stmt = conn.prepare("SELECT * FROM direct_messages WHERE (sender = ?1 AND receiver = ?2) OR (receiver = ?1 AND sender = ?2)").ok()?;
        let rows = stmt.query_map([name, target.clone()], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                sender: row.get(1)?,
                receiver: row.get(2)?,
                content: row.get(3)?,
                reference: row.get(4)?,
                conversation_id: row.get(5)?,
                time_created: row.get(6)?,
            })
        }).ok()?;
    
        let mut direct_messages: Vec<Conversation> = Vec::new();

        for dm in rows {
            direct_messages.push(dm.ok()?);
        }
    
        Some(Template::render("conversation", context! {
            messages: direct_messages,
            target: target,
        }))
    }

    else {
        Some(Template::render("login", context! {}))
    }
}

#[post("/conversations", data = "<message>")]
fn new_direct_message(cookies: &CookieJar<'_>, message: Form<DirectMessage>) -> Option<Redirect> {
    let name = cookies.get("username");
    let conn = Connection::open("forum.sqlite").ok()?;
    
    let id = retrieve_conversation_id(&cookies, &message.receiver.to_string())?;

    conn.execute(
        "INSERT INTO direct_messages (sender, receiver, content, conversation_id) values (?1, ?2, ?3, ?4)",
        [&name?.value().to_string(), &message.receiver.to_string(), &message.content.to_string(), &id.to_string()]
    ).ok()?;

    Some(Redirect::to("conversations"))
}

#[post("/conversations/<target>", data = "<message>", rank = 1)]
fn direct_message(cookies: &CookieJar<'_>, message: Form<String>, target: String) -> Option<Redirect> {
    let name = cookies.get("username");
    let conn = Connection::open("forum.sqlite").ok()?;

    let id = retrieve_conversation_id(&cookies, &target)?;

    conn.execute(
        "INSERT INTO direct_messages (sender, receiver, content, conversation_id) values (?1, ?2, ?3, ?4)",
        [&name?.value().to_string(), &target, &message.to_string(), &id.to_string()]
    ).ok()?;

    Some(Redirect::to(format!("/conversations/{}", target)))
}

#[get("/conversations")]
fn render_conversation_list(cookies: &CookieJar<'_>) -> Option<Template> {
    let name = cookies.get("username");
    let conn = Connection::open("forum.sqlite").ok()?;

    let mut stmt = conn.prepare("SELECT id FROM conversation_members WHERE user = ?1").ok()?;
    let rows = stmt.query_map([name?.value()], |row| {
        Ok(ID(row.get("id")?))
    }).ok()?;

    let mut conversation_partners: Vec<String> = Vec::new();

    for row in rows {
        let mut stmt = conn.prepare("SELECT user FROM conversation_members WHERE id = ?1 AND user != ?2").ok()?;
        let partners = stmt.query_map([&row.ok()?.0.to_string(), name?.value()], |row| {
            Ok(row.get("user")?)
        }).ok()?;

        for i in partners {
            conversation_partners.push(i.ok()?);
        }
    }

    Some(Template::render("conversation_list", context! {
        users: conversation_partners,
    }))
}

/* 
This function tests whether the user is logged in. 

if the user is logged in, it returns true. If not, false.
*/
fn logged_in(cookies: &CookieJar<'_>) -> bool {
    let id = cookies.get("user_id");

    if !id.is_some() {
        false
    }

    else {
        true
    }
}

fn retrieve_conversation_id(cookies: &CookieJar<'_>, target: &String) -> Option<i32> {
    let name = cookies.get("username");
    let conn = Connection::open("forum.sqlite").ok()?;

    let mut found_id: i32 = 0;

    let mut stmt = conn.prepare("SELECT id FROM conversation_members WHERE user = ?1").ok()?;
    let rows = stmt.query_map([name?.value()], |row| {
        Ok(ID(row.get("id")?))
    }).ok()?;

    let mut logged_in_ids: Vec<ID> = Vec::new();

    for x in rows {
       logged_in_ids.push(x.ok()?);
    }

    let mut stmt = conn.prepare("SELECT id FROM conversation_members WHERE user = ?1").ok()?;
    let rows = stmt.query_map([target], |row| {
        Ok(ID(row.get("id")?))
    }).ok()?;

    let mut target_ids: Vec<ID> = Vec::new();

    for x in rows {
       target_ids.push(x.ok()?);
    }

    for i in logged_in_ids {
        for j in &target_ids {
            if i.0 == j.0 {
                found_id = i.0;
            }
        }
    }

    if found_id != 0 {
        return Some(found_id);
    }

    else {
        let mut stmt = conn.prepare("SELECT MAX(id) FROM conversation_members").ok()?;

        let rows = stmt.query_map([], |row| {
            Ok(ID(row.get(0)?))
        }).ok()?;
        
        let mut temp: Vec<ID> = Vec::new();
        
        for x in rows {
            temp.push(x.ok()?);
        }
        
        let mut max_id: i32;
    
        if temp.is_empty() {
            max_id = 1
        }
    
        else {
            max_id = temp[0].0 + 1;
        }
        
        conn.execute(
            "INSERT INTO conversation_members (id, user) values (?1, ?2)",
            [max_id.to_string(), name?.value().to_string()]
        ).ok()?;
        
        conn.execute(
            "INSERT INTO conversation_members (id, user) values (?1, ?2)",
            [max_id.to_string(), target.to_string()]
        ).ok()?;

        Some(max_id)
    }
}

/*
Launch.
*/
#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/", routes![index, render_login, render_register, login, profile, submit, messages, logout, create_new_account, reply, render_conversation, direct_message, new_direct_message, render_conversation_list])
        .mount("/message", routes![render_message])
}

