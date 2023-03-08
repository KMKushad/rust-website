#[macro_use] extern crate rocket;

use rocket::tokio::time::{sleep, Duration};
use rocket::fs::NamedFile;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use rocket_dyn_templates::{Template, tera::Tera, context};

#[get("/")]
fn index() -> Template {
    Template::render("index", context! {
        title: "Hello"
    })
}

#[get("/delay/<seconds>")]
async fn delay(seconds: u64) -> String {
    sleep(Duration::from_secs(seconds)).await;
    format!("Waited for {} seconds", seconds)
}

#[get("/greet/<name>")]
fn greet(name: &str) -> String {
    format!("Hello, {}", name)
}

#[get("/upload")]
fn upload() -> String {
    let path = Path::new("test.txt");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(_) => (),
    };

    s
}

#[get("/<file..>")]
async fn files(file: std::path::PathBuf) -> Option<NamedFile> {
    NamedFile::open(std::path::Path::new("static/").join(file)).await.ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![files, index])
        .mount("/other_routes", routes![index, delay, greet, upload])
}

