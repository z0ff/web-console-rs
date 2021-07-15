use std::time::{Duration, Instant};
use std::process::Command;
//use once_cell::sync::OnceCell;
//use std::lazy::OnceCell;
use actix::prelude::*;
use actix_files::NamedFile;
use actix_web::{get, web, App, HttpRequest, HttpServer, Responder, Result};
use actix_web_actors::ws;
use sys_info::*;

//static outstr: OnceCell<String> = OnceCell::new();


#[get("/")]
async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}

#[get("/sh/{cmd}")]
async fn sh(web::Path(cmd): web::Path<String>) -> impl Responder{
    let output = Command::new("bash").arg("-c").arg(&cmd).output().expect("Failed to excecute command");
    //outstr.set(format!("$ {}\n{}", &cmd, String::from_utf8_lossy(&output.stdout))).unwrap();
    //format!("{}", outstr.get().unwrap())
    format!("$ {}\n{}", &cmd, String::from_utf8_lossy(&output.stdout))
}

#[get("/info")]
async fn info() -> impl Responder{
    format!("os: {} {}", os_type().unwrap(), os_release().unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(sh)
            .service(info)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}