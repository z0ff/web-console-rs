use actix::prelude::*;
use actix_files::NamedFile;
use actix_web::{
    get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use actix_web_actors::ws;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio_util::codec::{FramedRead, LinesCodec};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

struct MyWS {
    hb: Instant,
}

#[derive(Serialize, Deserialize, Debug)]
struct Script {
    lines: Vec<String>,
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
            //Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Text(text)) => {
                //#[must_use = "futures do nothing unless you `.await` or poll them"]
                exec_script(text, ctx);
                //ctx.text(text);
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
pub async fn script_index() -> Result<NamedFile> {
    Ok(NamedFile::open("static/script.html")?)
}

pub async fn script_start(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let res = ws::start(MyWS::new(), &req, stream);
    res
}

async fn exec_script(script_str: String, ctx: &mut <MyWS as Actor>::Context) {
    let script: Script = serde_json::from_str(script_str.trim()).unwrap();
    for ln in &script.lines {
        let mut child = Command::new("bash")
            .arg("-c")
            .arg(ln)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed");

        let stdout = child.stdout.take().unwrap();

        let mut reader = FramedRead::new(stdout, LinesCodec::new());

        while let Some(line) = reader.next().await {
            //println!("{}", line.unwrap());
            ctx.text(line.unwrap());
        }
    }
}
