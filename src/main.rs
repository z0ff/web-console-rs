pub mod script;

use crate::script::*;

use std::thread;
use std::time::Duration;
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
use systemstat::*;

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
    cpu_user: f32,
    cpu_nice: f32,
    cpu_system: f32,
    cpu_idle: f32,
    load_one: f32,
    load_five: f32,
    load_fifteen: f32,
    mem_total: u64,
    mem_free: u64,
}

#[get("/")]
async fn index(req: HttpRequest) -> Result<NamedFile> {
    println!("{:?}", req);
    let ua = req.headers().get("user-agent").unwrap().to_str().unwrap();
    let open_file: String;
    if ua.find("Trident") != None || ua.find("MSIE") != None {
        open_file = "static/ie.html".to_string();
    } else {
        open_file = "static/index.html".to_string();
    }
    Ok(NamedFile::open(open_file)?)
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

#[get("/status")]
async fn status() -> HttpResponse {
    let sys = systemstat::System::new();

    let mut cpu_user: f32 = 0.0;
    let mut cpu_nice: f32 = 0.0;
    let mut cpu_system: f32 = 0.0;
    let mut cpu_idle: f32 = 0.0;
    let mut load_one: f32 = 0.0;
    let mut load_five: f32 = 0.0;
    let mut load_fifteen: f32 = 0.0;
    let mut mem_total: u64 = 0;
    let mut mem_free: u64 = 0;

    match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            thread::sleep(Duration::from_secs(1));
            let cpu = cpu.done().unwrap();
            cpu_user = cpu.user;
            cpu_nice = cpu.nice;
            cpu_system = cpu.system;
            cpu_idle = cpu.idle;
        }
        Err(x) => println!("CPU load error: {}", x)
    }

    match sys.load_average() {
        Ok(load) => {
            load_one = load.one;
            load_five = load.five;
            load_fifteen = load.fifteen;
        },
        Err(x) => println!("Load average error: {}", x)
    }

    match sys.memory() {
        Ok(mem) => {
            mem_total = mem.total.0;
            mem_free = mem.free.0;
        }
        Err(x) => println!("Memory Error: {}", x)
    }

    HttpResponse::Ok().json(Status {
        os_type: os_type().unwrap(),
        os_release: os_release().unwrap(),
        cpu_num: cpu_num().unwrap(),
        cpu_speed: cpu_speed().unwrap(),
        proc_total: proc_total().unwrap(),
        cpu_user: cpu_user,
        cpu_nice: cpu_nice,
        cpu_system: cpu_system,
        cpu_idle: cpu_idle,
        load_one: load_one,
        load_five: load_five,
        load_fifteen: load_fifteen,
        mem_total: mem_total,
        mem_free: mem_free,
    })
}

async fn not_found() -> Result<NamedFile> {
    Ok(NamedFile::open("static/404.html")?)
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

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(status)
            .service(postcmd)
            .service(script_index)
            .service(web::resource("/script/ws/").route(web::get().to(script_start))
        )
        .default_service(
            web::route().to(not_found)
        )
    })
        .bind_openssl("127.0.0.1:8080", builder)?
        .run()
        .await
}
