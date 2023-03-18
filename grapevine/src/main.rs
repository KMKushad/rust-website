#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket::response::Redirect;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket_dyn_templates::{Template, context};
use either::*;
use serde::{Deserialize, Serialize};
use rusqlite::{Connection};
type Session<'a> = rocket_session::Session<'a, User>;

#[derive(Debug, FromForm)]
struct AccountInfo {
    username: String,
    password: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    password: String,
    time_created: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct Message {
    id: i32,
    content: String,
    time_created: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct ID(i32);

#[get("/profile")]
fn profile(cookies: &CookieJar<'_>) -> Either<Redirect, Template> {
    let id = cookies.get("user_id");
    let name = cookies.get("username");

    if id.is_some() {
        Right(
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

#[get("/")]
fn index() -> Template {
    Template::render("hello", context! {
        title: String::from("Hello")
    })
}

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

    println!("{:?}", ids[0].0);

    cookies.add(Cookie::new("user_id", ids[0].0.to_string()));
    cookies.add(Cookie::new("username", credentials.username.clone()));

    Some(Left(Redirect::to("profile")))
}   

#[get("/register")]
fn render_register(cookies: &CookieJar<'_>) -> Either<Template, Redirect> {
    let id = cookies.get("user_id");

    if !id.is_some() {
        Left(Template::render("register", context! {}))
    }

    else {
        Right(Redirect::to("profile"))
    }
}

#[get("/login")]
fn render_login(cookies: &CookieJar<'_>) -> Either<Template, Redirect> {
    let id = cookies.get("user_id");

    if !id.is_some() {
        Left(Template::render("login", context! {}))
    }

    else {
        Right(Redirect::to("profile"))
    }
}

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

#[get("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Template {
    cookies.remove(Cookie::named("user_id"));
    cookies.remove(Cookie::named("username"));

    Template::render("hello", context!{
        title: "Successfully logged out."
    })
}

#[post("/message", data = "<message>")]
fn submit(message: Form<&str>) -> Option<Redirect> {
    let conn = Connection::open("forum.sqlite").ok()?;

    conn.execute(
        "INSERT INTO messages (content) values (?1)",
        [message.to_string()]
    ).ok()?;

    Some(Redirect::to("message"))
}

#[get("/message")]
fn messages() -> Option<Template> {
    let conn = Connection::open("forum.sqlite").ok()?;

    let mut stmt = conn.prepare("SELECT id, content, time_posted FROM messages").ok()?;
    let rows = stmt.query_map([], |row| {
        Ok(Message {
            id: row.get(0)?,
            content: row.get(1)?,
            time_created: row.get(2)?,
        })
    }).ok()?;

    let mut messages: Vec<Message> = Vec::new();

    for message in rows {
        messages.push(message.ok()?);
    }

    println!("{:?}", messages);

    Some(Template::render("message", context! {
        messages: messages,
    }))
}


#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .attach(Session::fairing())
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/", routes![index, render_login, render_register, login, profile, submit, messages, logout, create_new_account])
}

