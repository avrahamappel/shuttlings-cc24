use actix_web::http::header;
use actix_web::{get, web::ServiceConfig, HttpResponse};
use shuttle_actix_web::ShuttleActixWeb;

#[get("/")]
async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

#[get("/-1/seek")]
async fn rick_roll() -> HttpResponse {
    HttpResponse::Found()
        .insert_header((
            header::LOCATION,
            "https://www.youtube.com/watch?v=9Gc4QTqslN4",
        ))
        .finish()
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(hello_bird).service(rick_roll);
    };

    Ok(config.into())
}
