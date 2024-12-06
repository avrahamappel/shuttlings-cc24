use std::net::{Ipv4Addr, Ipv6Addr};

use actix_web::http::header;
use actix_web::web::Query;
use actix_web::{get, web::ServiceConfig, HttpResponse};
use serde::Deserialize;
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

#[derive(Deserialize)]
struct DestQueryParams {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

#[get("/2/dest")]
async fn day2part1(params: Query<DestQueryParams>) -> String {
    let parts: Vec<_> = params
        .from
        .octets()
        .iter()
        .enumerate()
        .map(|(i, o)| o.overflowing_add(params.key.octets()[i]).0.to_string())
        .collect();

    parts.join(".")
}

#[derive(Deserialize)]
struct KeyQueryParams {
    from: Ipv4Addr,
    to: Ipv4Addr,
}

#[get("/2/key")]
async fn day2part2(params: Query<KeyQueryParams>) -> String {
    let parts: Vec<_> = params
        .to
        .octets()
        .iter()
        .enumerate()
        .map(|(i, o)| o.overflowing_sub(params.from.octets()[i]).0.to_string())
        .collect();

    parts.join(".")
}

#[derive(Deserialize)]
struct V6DestQueryParams {
    from: Ipv6Addr,
    key: Ipv6Addr,
}

#[get("/2/v6/dest")]
async fn day2part3dest(params: Query<V6DestQueryParams>) -> String {
    let parts: Vec<_> = params
        .from
        .octets()
        .iter()
        .enumerate()
        .map(|(i, o)| o ^ params.key.octets()[i])
        .collect();

    let parts: [u8; 16] = parts.try_into().unwrap();
    Ipv6Addr::from(parts).to_string()
}

#[derive(Deserialize)]
struct V6KeyQueryParams {
    from: Ipv6Addr,
    to: Ipv6Addr,
}

#[get("/2/v6/key")]
async fn day2part3key(params: Query<V6KeyQueryParams>) -> String {
    let parts: Vec<_> = params
        .to
        .octets()
        .iter()
        .enumerate()
        .map(|(i, o)| (o ^ params.from.octets()[i]))
        .collect();

    let parts: [u8; 16] = parts.try_into().unwrap();
    Ipv6Addr::from(parts).to_string()
}

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(hello_bird)
            .service(rick_roll)
            .service(day2part1)
            .service(day2part2)
            .service(day2part3dest)
            .service(day2part3key);
    };

    Ok(config.into())
}
