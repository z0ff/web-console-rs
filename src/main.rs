pub mod script;

use crate::script::*;

use std::future::{ready, Ready};
use std::process::Command;
//use std::time::{Duration, Instant};
//use once_cell::sync::OnceCell;
//use std::lazy::OnceCell;
//use actix::prelude::*;
use actix_files::NamedFile;
use actix_web::{
    get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
//use actix_web_actors::ws;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::{Deserialize, Serialize};
use sys_info::*;

//static outstr: OnceCell<String> = OnceCell::new();

#[derive(Deserialize)]
struct Cmd {
    cmd: String,
}

#[derive(Serialize)]
struct Res {
    res: String,
}

impl Responder for Res {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

#[derive(Serialize, Deserialize)]
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
async fn index(req: HttpRequest) -> Result<NamedFile> {
    println!("{:?}", req);
    Ok(NamedFile::open("static/index.html")?)
}

#[post("/postcmd")]
async fn postcmd(cmd: web::Json<Cmd>) -> impl Responder {
    let output = Command::new("bash")
        .arg("-c")
        .arg(&cmd.cmd)
        .output()
        .expect("Failed to excecute command");
    Res {
        res: String::from_utf8_lossy(&output.stdout).into_owned(),
    }
}

/*
#[get("/getcmd")]
async fn getres() -> HttpResponse {
    HttpResponse::Ok().json(Res {
        res: "response".to_string(),
    })
}
*/

#[get("/status")]
async fn status() -> HttpResponse {
    let load = loadavg().unwrap();
    let mem = mem_info().unwrap();
    HttpResponse::Ok().json(Status {
        os_type: os_type().unwrap(),
        os_release: os_release().unwrap(),
        cpu_num: cpu_num().unwrap(),
        cpu_speed: cpu_speed().unwrap(),
        proc_total: proc_total().unwrap(),
        load_one: load.one,
        load_five: load.five,
        load_fifteen: load.fifteen,
        mem_total: mem.total,
        mem_free: mem.free,
        mem_avail: mem.avail,
        mem_buffers: mem.buffers,
        mem_cached: mem.cached,
        mem_swap_total: mem.swap_total,
        mem_swap_free: mem.swap_free,
    })
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // load ssl keys
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(|| App::new().service(index).service(status).service(postcmd).service(script_index))
        .bind_openssl("127.0.0.1:8080", builder)?
        .run()
        .await
}
