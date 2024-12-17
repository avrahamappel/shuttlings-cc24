use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Mutex;

use actix_web::http::header;
use actix_web::web::{Data, Header, Json, Query, ServiceConfig};
use actix_web::{get, post, Either, HttpRequest, HttpResponse};
use cargo_toml::ContentType;
use serde::Deserialize;
use shuttle_actix_web::ShuttleActixWeb;

mod bucket;
mod cargo_toml;
mod conversion;

use bucket::Bucket;
use cargo_toml::CargoOrders;
use conversion::Conversion;

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

#[post("/5/manifest")]
async fn day5(data: String, request: HttpRequest) -> HttpResponse {
    let content_type = request.headers().get("Content-type");

    if content_type.is_none()
        || !matches!(
            content_type.unwrap().as_bytes(),
            b"application/json" | b"application/yaml" | b"application/toml",
        )
    {
        return HttpResponse::UnsupportedMediaType().finish();
    }
    let content_type = match content_type.unwrap().as_bytes() {
        b"application/json" => ContentType::Json,
        b"application/yaml" => ContentType::Yaml,
        b"application/toml" => ContentType::Toml,
        _ => unreachable!(),
    };

    match cargo_toml::from_str(&data, content_type) {
        CargoOrders::Orders(orders) => {
            if orders.is_empty() {
                HttpResponse::NoContent().finish()
            } else {
                let order_str = orders
                    .iter()
                    .map(|o| format!("{}: {}", o.item, o.quantity))
                    .collect::<Vec<_>>()
                    .join("\n");
                HttpResponse::Ok().body(order_str)
            }
        }
        CargoOrders::KeywordMissing => {
            HttpResponse::BadRequest().body("Magic keyword not provided")
        }
        CargoOrders::InvalidManifest => HttpResponse::BadRequest().body("Invalid manifest"),
    }
}

#[post("/9/milk")]
async fn day9(
    bucket: Data<Mutex<Bucket>>,
    content_type: Option<Header<header::ContentType>>,
    body: Option<Json<Conversion>>,
) -> Either<Json<Conversion>, HttpResponse> {
    if bucket.lock().unwrap().get_milk() {
        match content_type {
            Some(Header(header::ContentType(mime))) if mime.essence_str() == "application/json" => {
                if let Some(conversion) = body {
                    Either::Left(Json(conversion.convert()))
                } else {
                    Either::Right(HttpResponse::BadRequest().finish())
                }
            }
            _ => Either::Right(HttpResponse::Ok().body("Milk withdrawn\n")),
        }
    } else {
        Either::Right(HttpResponse::TooManyRequests().body("No milk available\n"))
    }
}

#[post("/9/refill")]
async fn day9refill(bucket: Data<Mutex<Bucket>>) -> HttpResponse {
    bucket.lock().unwrap().refill();
    HttpResponse::Ok().finish()
}

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let bucket = Data::new(Mutex::new(Bucket::new())).clone();

    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(bucket)
            .service(hello_bird)
            .service(rick_roll)
            .service(day2part1)
            .service(day2part2)
            .service(day2part3dest)
            .service(day2part3key)
            .service(day5)
            .service(day9)
            .service(day9refill);
    };

    Ok(config.into())
}
