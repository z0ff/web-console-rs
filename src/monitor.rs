use std::thread;
use std::time::Duration;
use actix_web::{get, HttpResponse};
use serde::{Deserialize, Serialize};
use sys_info::*;
use systemstat::*;

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
