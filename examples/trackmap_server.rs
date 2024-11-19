use actix::AsyncContext;
use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json;
use std::borrow::Cow;

// WebSocket message types
#[derive(Serialize, Deserialize, Debug)]
struct MapCommand {
    command: String,
    lat: f64,
    lng: f64,
    zoom: Option<f64>,
}

// WebSocket actor
struct MapSocket {
    heartbeat: Option<actix::SpawnHandle>,
}

impl Actor for MapSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let handle = ctx.run_interval(std::time::Duration::from_secs(5), |act, ctx| {
            let command = MapCommand {
                command: "fly_to".to_string(),
                // range of lat/lng for roughly mainland PRC
                lat: rand::thread_rng().gen_range(30.0..53.0),
                lng: rand::thread_rng().gen_range(73.0..135.0),
                zoom: Some(rand::thread_rng().gen_range(3.0..12.0)),
            };

            println!("Sending command: {:?}", command);

            if let Ok(msg) = serde_json::to_string(&command) {
                ctx.text(msg);
            }
        });
        self.heartbeat = Some(handle);
    }
}

impl Default for MapSocket {
    fn default() -> Self {
        Self { heartbeat: None }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MapSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                println!("Received message: {}", text);
                // Handle incoming messages if needed
            }
            _ => (),
        }
    }
}

// WebSocket connection handler
async fn ws_route(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MapSocket::default(), &req, stream)
}

// Serve the HTML page
async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../static/tracks-client.html"))
}

const FOGCORE_JS: &str = include_str!("../pkg/fogcore.js");
const FOGCORE_WASM: &[u8] = include_bytes!("../pkg/fogcore_bg.wasm");

const TRACKS_LAYER_JS: &str = include_str!("../static/tracks-layer.js");

const TILES_DATA: &[u8] = include_bytes!("../static/tiles.zip");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://localhost:5503");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/ws", web::get().to(ws_route))
            .route("/fogcore.js", web::get().to(serve_fogcore_js))
            .route("/fogcore_bg.wasm", web::get().to(serve_fogcore_wasm))
            .route("/tracks-layer.js", web::get().to(serve_tracks_layer_js))
            .route("/tiles.zip", web::get().to(serve_tiles_zip))
    })
    .bind("localhost:5503")?
    .run()
    .await
}

async fn serve_fogcore_js() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(FOGCORE_JS)
}

async fn serve_fogcore_wasm() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/wasm")
        .body(Cow::Borrowed(FOGCORE_WASM))
}

async fn serve_tracks_layer_js() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(TRACKS_LAYER_JS)
}

async fn serve_tiles_zip() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/zip")
        .body(Cow::Borrowed(TILES_DATA))
}
