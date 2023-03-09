#[macro_use] extern crate rocket;

//use rocket::response::content::RawHtml;
use rocket::fs::{FileServer, relative};

//use rocket::Request;
//use rocket::response::Redirect;

use rocket_dyn_templates::{Template, context};

use std::io;

#[get("/")]
fn index() -> Template {
    Template::render("hello", context! {
        title: "Hello"
    })
}

#[get("/profile")]
fn profile() -> Template {
    let mut username = String::new();

    io::stdin()
        .read_line(&mut username)
        .expect("Failed to read line");

    Template::render("profile", context! {
        username,
        email: "test@gmail.com"
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/", routes![index, profile])
}

