use std::process::Command;
use std::time::{Duration, Instant};
//use once_cell::sync::OnceCell;
//use std::lazy::OnceCell;
use actix::prelude::*;
use actix_files::NamedFile;
use actix_web::{get, web, App, HttpRequest, HttpServer, Responder, Result};
use actix_web_actors::ws;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::Deserialize;
use sys_info::*;

//static outstr: OnceCell<String> = OnceCell::new();

#[derive(Deserialize)]
struct Status {
    os_type: String,
    os_release: String,
    cpu_num: u32,
    cpu_speed: u64,
    proc_total: u64,
    load_one: f64,
    load_five: f64,
    load_fifteen: f64,
    mem_total: u64,
    mem_free: u64,
    mem_avail: u64,
    mem_buffers: u64,
    mem_cached: u64,
    mem_swap_total: u64,
    mem_swap_free: u64,
}

#[get("/")]
async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}

#[get("/sh/{cmd}")]
async fn sh(web::Path(cmd): web::Path<String>) -> impl Responder {
    let output = Command::new("bash")
        .arg("-c")
        .arg(&cmd)
        .output()
        .expect("Failed to excecute command");
    //outstr.set(format!("$ {}\n{}", &cmd, String::from_utf8_lossy(&output.stdout))).unwrap();
    //format!("{}", outstr.get().unwrap())
    format!("$ {}\n{}", &cmd, String::from_utf8_lossy(&output.stdout))
}

#[get("/status")]
async fn status() -> impl Responder {
    format!("os: {} {}", os_type().unwrap(), os_release().unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // load ssl keys
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(|| App::new().service(index).service(sh).service(status))
        .bind_openssl("127.0.0.1:8080", builder)?
        .run()
        .await
}
