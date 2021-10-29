extern crate diesel;

use self::diesel::prelude::*;
use self::models::*;

use actix::prelude::*;
use actix::AsyncContext;
use actix_files::NamedFile;
use actix_web::{
    get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use actix_web_actors::ws;
//use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::stream::{StreamExt};
use tokio_util::codec::{FramedRead, LinesCodec};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

struct MyWS {
    hb: Instant,
}

#[derive(Serialize, Deserialize, Debug)]
struct Script {
    //user: String,
    lines: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct OutLn2 {
    line: String,
}
impl Handler<OutLn2> for MyWS {
    type Result = ();

    fn handle(&mut self, msg: OutLn2, ctx: &mut Self::Context) {
        ctx.text(msg.line);
    }
}

impl Actor for MyWS {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWS {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("WS: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                let rec = ctx.address().recipient();
                let fut = async move {
                    let script: Script = serde_json::from_str(text.trim()).unwrap();
                    let mut child = Command::new("bash")
                        .arg("-c")
                        .arg(&script.lines)
                        .stdout(Stdio::piped())
                        .spawn()
                        .expect("Failed to execute command");

                    let stdout = child.stdout.take().unwrap();

                    let mut reader = FramedRead::new(stdout, LinesCodec::new());

                    while let Some(line) = reader.next().await {
                        //println!("{}", line.unwrap());
                        rec.do_send(OutLn2 { line:line.unwrap() } ).expect("Failed to send stdout.");
                    }
                };
                fut.into_actor(self).spawn(ctx);
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl MyWS {
    fn new() -> Self {
        Self { hb: Instant::now() }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

#[get("/script")]
pub async fn script_queue_index() -> Result<NamedFile> {
    Ok(NamedFile::open("static/script.html")?)
}

pub async fn script_queue_start(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let res = ws::start(MyWS::new(), &req, stream);
    res
}
