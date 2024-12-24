use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Mutex;

use actix_web::cookie::Cookie;
use actix_web::http::header;
use actix_web::web::{Data, Header, Json, Query, ServiceConfig};
use actix_web::{get, post, Either, HttpRequest, HttpResponse};
use cargo_toml::ContentType;
use jwt_simple::{prelude::*, JWTError};
use serde::Deserialize;
use serde_json::Value;
use shuttle_actix_web::ShuttleActixWeb;

mod bucket;
mod cargo_toml;
mod conversion;
mod game;
mod quote_book;

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
async fn day5(data: String, content_type: Header<header::ContentType>) -> HttpResponse {
    let content_type = match content_type.0 .0.essence_str() {
        "application/json" => ContentType::Json,
        "application/yaml" => ContentType::Yaml,
        "application/toml" => ContentType::Toml,
        _ => {
            return HttpResponse::UnsupportedMediaType().finish();
        }
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

type JWTKey = Data<HS256Key>;

#[post("/16/wrap")]
async fn day16part1wrap(key: JWTKey, json: Json<Value>) -> HttpResponse {
    let jwt = key
        .authenticate(Claims::with_custom_claims(
            json.into_inner(),
            Duration::from_mins(5),
        ))
        .expect("key should be valid");
    let cookie = Cookie::new("gift", jwt);
    let mut response = HttpResponse::Ok().finish();
    response
        .add_cookie(&cookie)
        .expect("adding cookie should be fine");
    response
}

#[get("/16/unwrap")]
async fn day16part1unwrap(key: JWTKey, request: HttpRequest) -> Either<Json<Value>, HttpResponse> {
    if let Some(cookie) = request.cookie("gift") {
        if let Ok(claim) = key.verify_token(cookie.value(), None) {
            return Either::Left(Json(claim.custom));
        }
    }
    Either::Right(HttpResponse::BadRequest().finish())
}

#[derive(Deserialize)]
struct JwtHeader {
    //typ: String,
    alg: String,
}

enum RSAPublicKey {
    RS256(RS256PublicKey),
    RS512(RS512PublicKey),
}

impl RSAPublicKey {
    fn new(jwt: &str) -> Option<Self> {
        let jwt_head_str = jwt.split_once('.')?.0;
        #[allow(deprecated)]
        let jwt_head: JwtHeader =
            serde_json::from_slice(&base64::decode(jwt_head_str).ok()?).ok()?;
        let pem = include_str!("../day16_santa_public_key.pem");
        let key = match jwt_head.alg.as_str() {
            "RS256" => Self::RS256(RS256PublicKey::from_pem(pem).unwrap()),
            "RS512" => Self::RS512(RS512PublicKey::from_pem(pem).unwrap()),
            _ => unimplemented!(),
        };
        Some(key)
    }

    fn verify_token(&self, token: &str) -> Result<JWTClaims<Value>, jwt_simple::Error> {
        match self {
            RSAPublicKey::RS256(rs256_public_key) => rs256_public_key.verify_token(token, None),
            RSAPublicKey::RS512(rs512_public_key) => rs512_public_key.verify_token(token, None),
        }
    }
}

#[post("/16/decode")]
async fn day16part2(jwt: String) -> Either<HttpResponse, Json<Value>> {
    dbg!(&jwt);
    if let Some(key) = RSAPublicKey::new(&jwt) {
        match key.verify_token(&jwt) {
            Ok(claim) => {
                dbg!(&claim.custom);
                return Either::Right(Json(claim.custom));
            }
            Err(err) => {
                dbg!(&err);
                if let Ok(JWTError::InvalidSignature) = err.downcast() {
                    return Either::Left(HttpResponse::Unauthorized().finish());
                }
            }
        }
    }
    Either::Left(HttpResponse::BadRequest().finish())
}

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: sqlx::PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let bucket = Data::new(Mutex::new(Bucket::new())).clone();
    let game = game::new_shared_game().clone();
    let rng = game::new_shared_rng().clone();
    let jwt_key = Data::new(HS256Key::generate()).clone();
    let db = quote_book::shared_db_pool(pool).await.clone();
    let page_cache = quote_book::shared_page_cache().clone();

    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(bucket)
            .app_data(game)
            .app_data(rng)
            .app_data(jwt_key)
            .app_data(db)
            .app_data(page_cache)
            .service(hello_bird)
            .service(rick_roll)
            .service(day2part1)
            .service(day2part2)
            .service(day2part3dest)
            .service(day2part3key)
            .service(day5)
            .service(day9)
            .service(day9refill)
            .service(game::scope())
            .service(day16part1wrap)
            .service(day16part1unwrap)
            .service(day16part2)
            .service(quote_book::scope())
            .service(actix_files::Files::new("/assets", "./assets"));
    };

    Ok(config.into())
}
