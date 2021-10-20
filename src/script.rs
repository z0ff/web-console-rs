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

#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
struct ScriptExecutor(String);

struct MyWS {
    hb: Instant,
}

#[derive(Serialize, Deserialize, Debug)]
struct Script {
    lines: String,
}

#[derive(Debug)]
struct OutLn(String);

impl StreamHandler<Result<OutLn, ws::ProtocolError>> for MyWS {
    fn handle(
        &mut self,
        msg: Result<OutLn, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(OutLn(line)) => ctx.text(line),
            _ => ()
        }
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
            //Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Text(text)) => {
                //#[must_use = "futures do nothing unless you `.await` or poll them"]
                //exec_script(text, ctx);
                //ctx.text(text);
                ctx.notify(ScriptExecutor(text.to_string()));
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

impl Handler<ScriptExecutor> for MyWS {
    type Result = Result<(), ()>;
    fn handle(&mut self, msg: ScriptExecutor, ctx: &mut Self::Context) -> Self::Result {
        let script: Script = serde_json::from_str(msg.0.trim()).unwrap();
        let mut child = Command::new("bash")
            .arg("-c")
            .arg(&script.lines)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failde");
        //let mut cmd = Command::new(script.lines);
        //cmd.stdout(Stdio::piped());
        //let mut child = cmd.spawn().expect("err");

        let stdout = child.stdout.take().unwrap();

        let reader = FramedRead::new(stdout, LinesCodec::new());
        //let reader = BufReader::new(stdout).lines();

        let fut = async move {
            let status = child.await.expect("err");
            println!("child status was: {}", status);
        };
        let fut = actix::fut::wrap_future::<_, Self>(fut);
        ctx.spawn(fut);
        ctx.add_stream(reader.map(|l| Ok(OutLn(l.expect("Not a line")))));
        Ok(())
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
