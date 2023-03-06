#[macro_use] extern crate rocket;
use rocket::tokio::time::{sleep, Duration};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
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

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, delay, greet, upload])
}


