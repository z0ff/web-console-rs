use actix::prelude::*;
use actix::AsyncContext;
use actix_web::{
    get, post, web, Error, HttpRequest, HttpResponse, Result,
};
use actix_web_actors::ws;
use actix_redis::{Command as RCmd, RedisActor};
//use futures::prelude::*;
use redis_async::{resp::RespValue, resp_array};
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
pub struct OutLn {
    line: String,
}
impl Handler<OutLn> for MyWS {
    type Result = ();

    fn handle(&mut self, msg: OutLn, ctx: &mut Self::Context) {
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
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError,>, ctx: &mut Self::Context) {
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
                        rec.do_send(OutLn { line:line.unwrap() } ).expect("Failed to send stdout.");
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

async fn enqueue_job(script: String, redis: web::Data<Addr<RedisActor>>) -> Result<HttpResponse, Error> {
    let res = redis.send(RCmd(resp_array!["RPUSH", "jobQueue", script])).await?;
    match res {
        Ok(RespValue::Integer(_)) => {
            Ok(HttpResponse::Ok().body("Successfully enqueued job"))
        }
        _ => {
            println!("---->{:?}", res);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }

}

pub async fn script_start(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let res = ws::start(MyWS::new(), &req, stream);
    res
}
