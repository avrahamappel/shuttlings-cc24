use std::net::{Ipv4Addr, Ipv6Addr};

use actix_web::http::header;
use actix_web::web::{Query, ServiceConfig};
use actix_web::{get, post, HttpResponse};
use serde::{Deserialize, Deserializer};
use shuttle_actix_web::ShuttleActixWeb;
use toml::Value;

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

#[derive(Deserialize, Debug)]
struct Order {
    item: String,
    quantity: u32,
}

fn deserialize_orders<'de, D>(des: D) -> Result<Option<Vec<Order>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<Vec<Value>> = Option::deserialize(des)?;

    if let Some(values) = opt {
        let mut result = Vec::new();

        for value in values {
            if let Ok(inner) = value.try_into() {
                result.push(inner);
            }
        }

        Ok(Some(result))
    } else {
        Ok(None)
    }
}

#[derive(Deserialize, Debug)]
struct Metadata {
    #[serde(default, deserialize_with = "deserialize_orders")]
    orders: Option<Vec<Order>>,
}

#[derive(Deserialize, Debug)]
struct WrappedMetadata {
    metadata: Metadata,
}

fn deserialize_metadata<'de, D>(des: D) -> Result<Metadata, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum MaybeWrappedMetadata {
        Wrapped(WrappedMetadata),
        Unwrapped(Metadata),
    }

    let deserialized = MaybeWrappedMetadata::deserialize(des)?;
    match deserialized {
        MaybeWrappedMetadata::Wrapped(wm) => Ok(wm.metadata),
        MaybeWrappedMetadata::Unwrapped(m) => Ok(m),
    }
}

#[derive(Deserialize, Debug)]
struct Package {
    name: String,
    authors: Option<Vec<String>>,
    keywords: Vec<String>,
    // one of the tests has double nesting by mistake
    #[serde(alias = "package", deserialize_with = "deserialize_metadata")]
    metadata: Metadata,
}

#[derive(Deserialize, Debug)]
struct CargoToml {
    package: Package,
}

#[post("/5/manifest")]
async fn day5part1(data: String) -> HttpResponse {
    dbg!(&data);
    dbg!(toml::from_str::<toml::Table>(&data).unwrap());
    if let Ok(toml) = toml::from_str::<CargoToml>(&data) {
        let maybe_orders = toml.package.metadata.orders;

        if let Some(orders) = maybe_orders {
            if !orders.is_empty() {
                let order_str = orders
                    .iter()
                    .map(|o| format!("{}: {}", o.item, o.quantity))
                    .collect::<Vec<_>>()
                    .join("\n");

                eprintln!("ok");
                return HttpResponse::Ok().body(order_str);
            }
        }

        eprintln!("empty");
        HttpResponse::NoContent().finish()
    } else {
        eprintln!("bad request");
        HttpResponse::BadRequest().body("Invalid manifest")
    }
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
            .service(day2part3key)
            .service(day5part1);
    };

    Ok(config.into())
}
