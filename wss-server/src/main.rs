use actix::prelude::*;
use actix::AsyncContext;
use actix_web::get;
use actix_web::web::Payload;
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::time::{Duration, Instant};

#[get("/ws")]
async fn websocket_handler(req: HttpRequest, stream: Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWebSocket::new(), &req, stream)
}

struct MyWebSocket {
    hb: Instant,
}

impl MyWebSocket {
    fn new() -> Self {
        Self { hb: Instant::now() }
    }
}

impl actix::Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl MyWebSocket {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(5), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::from_secs(10) {
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("Received from client: {}", text);
                ctx.text(format!("Hello from server! You said: {}", text));
            }
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Close(reason)) => {
                println!("WebSocket closed: {:?}", reason);
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting secure WSS server at https://localhost:8080");

    // ✅ unwrap the Result before calling methods on it
    let mut builder =
        SslAcceptor::mozilla_intermediate(SslMethod::tls()).expect("Failed to create TLS builder");

    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .expect("Failed to set private key");
    builder
        .set_certificate_chain_file("cert.pem")
        .expect("Failed to set certificate chain");

    HttpServer::new(|| App::new().service(websocket_handler))
        .bind_openssl("0.0.0.0:8080", builder)?
        .run()
        .await
}
