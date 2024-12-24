use std::collections::HashMap;
use std::str::FromStr;

use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, post, put, Either, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::game::SharedRng;

type SharedDBPool = Data<PgPool>;

pub async fn shared_db_pool(pool: PgPool) -> SharedDBPool {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Migration was executed successfully");

    Data::new(pool)
}

#[derive(Debug, Serialize, FromRow)]
struct Quote {
    id: Uuid,
    author: String,
    #[allow(clippy::struct_field_names)]
    quote: String,
    created_at: DateTime<Utc>,
    version: i32,
}

impl Quote {
    async fn find(pool: &PgPool, id: &Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM quotes WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }
}

#[derive(Debug, Deserialize)]
struct QuoteRequest {
    author: String,
    quote: String,
}

#[post("reset")]
async fn reset(pool: SharedDBPool) -> HttpResponse {
    sqlx::query("TRUNCATE TABLE quotes")
        .execute(&**pool)
        .await
        .expect("Couldn't truncate quotes table");

    HttpResponse::Ok().finish()
}

#[get("cite/{id}")]
async fn cite(id: Path<String>, pool: SharedDBPool) -> Either<HttpResponse, Json<Quote>> {
    let Ok(id) = Uuid::from_str(&id) else {
        return Either::Left(HttpResponse::BadRequest().finish());
    };
    let res = Quote::find(&pool, &id).await;

    match res {
        Ok(quote) => Either::Right(Json(quote)),
        Err(err) => {
            dbg!(err);
            Either::Left(HttpResponse::NotFound().finish())
        }
    }
}

#[delete("remove/{id}")]
async fn remove(id: Path<String>, pool: SharedDBPool) -> Either<HttpResponse, Json<Quote>> {
    let Ok(id) = Uuid::from_str(&id) else {
        return Either::Left(HttpResponse::BadRequest().finish());
    };
    let res = sqlx::query_as::<_, Quote>("DELETE FROM quotes WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_one(&**pool)
        .await;

    match res {
        Ok(quote) => Either::Right(Json(quote)),
        Err(err) => {
            dbg!(err);
            Either::Left(HttpResponse::NotFound().finish())
        }
    }
}

#[put("undo/{id}")]
async fn undo(
    id: Path<String>,
    pool: SharedDBPool,
    form: Json<QuoteRequest>,
) -> Either<HttpResponse, Json<Quote>> {
    let Ok(id) = Uuid::from_str(&id) else {
        return Either::Left(HttpResponse::BadRequest().finish());
    };
    let Ok(mut quote) = Quote::find(&pool, &id).await else {
        return Either::Left(HttpResponse::NotFound().finish());
    };

    quote.author.clone_from(&form.author);
    quote.quote.clone_from(&form.quote);
    quote.version += 1;

    let res = sqlx::query("UPDATE quotes SET author = $1, quote = $2, version = $3 WHERE id = $4")
        .bind(&quote.author)
        .bind(&quote.quote)
        .bind(quote.version)
        .bind(quote.id)
        .execute(&**pool)
        .await;

    match res {
        Ok(_) => Either::Right(Json(quote)),
        Err(err) => {
            dbg!(err);
            Either::Left(HttpResponse::InternalServerError().finish())
        }
    }
}

#[post("/draft")]
async fn draft(pool: SharedDBPool, form: Json<QuoteRequest>) -> HttpResponse {
    match sqlx::query_as::<_, Quote>(
        "INSERT INTO quotes (id, author, quote) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(&form.author)
    .bind(&form.quote)
    .fetch_one(&**pool)
    .await
    {
        Ok(quote) => HttpResponse::Created().json(quote),
        Err(err) => {
            dbg!(err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

type Token = String;
type Page = i64;
type SharedPageCache = Data<Mutex<HashMap<Token, Page>>>;

pub fn shared_page_cache() -> SharedPageCache {
    Data::new(Mutex::new(HashMap::new()))
}

#[derive(Debug, Deserialize)]
struct ListParams {
    token: Token,
}

#[derive(Debug, Serialize)]
struct ListResponse {
    quotes: Vec<Quote>,
    page: Page,
    next_token: Option<Token>,
}

fn random_string(rng: &mut StdRng) -> String {
    rng.sample_iter(rand::distributions::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

#[get("/list")]
async fn list(
    pool: SharedDBPool,
    cache: SharedPageCache,
    rng: SharedRng,
    query: Option<Query<ListParams>>,
) -> HttpResponse {
    const PAGESIZE: Page = 3;

    let count = match sqlx::query_scalar::<_, Page>("SELECT COUNT(*) FROM quotes")
        .fetch_one(&**pool)
        .await
    {
        Ok(count) => count,
        Err(err) => {
            dbg!(err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut pages = count / PAGESIZE;
    // Often the last page has less than PAGESIZE
    if count % PAGESIZE > 0 {
        pages += 1;
    }

    let page = if let Some(params) = query {
        let mut map = cache.lock().await;
        let Some(page) = map.remove(&params.token) else {
            return HttpResponse::BadRequest().finish();
        };
        page
    } else {
        1
    };

    let next_token = if pages > page {
        let mut rng = rng.lock().await;
        let token = random_string(&mut rng);
        let mut map = cache.lock().await;
        map.insert(token.clone(), page + 1);
        Some(token)
    } else {
        None
    };

    match sqlx::query_as::<_, Quote>("SELECT * FROM quotes ORDER BY created_at OFFSET $1 LIMIT $2")
        .bind((page - 1) * PAGESIZE)
        .bind(PAGESIZE)
        .fetch_all(&**pool)
        .await
    {
        Ok(quotes) => HttpResponse::Ok().json(ListResponse {
            quotes,
            page,
            next_token,
        }),
        Err(err) => {
            dbg!(err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub fn scope() -> Scope {
    Scope::new("/19")
        .service(reset)
        .service(cite)
        .service(remove)
        .service(undo)
        .service(draft)
        .service(list)
}
