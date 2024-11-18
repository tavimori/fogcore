use actix::AsyncContext;
use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json;

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
                lat: rand::thread_rng().gen_range(-90.0..90.0),
                lng: rand::thread_rng().gen_range(-180.0..180.0),
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://127.0.0.1:5503");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/ws", web::get().to(ws_route))
    })
    .bind("127.0.0.1:5503")?
    .run()
    .await
}
