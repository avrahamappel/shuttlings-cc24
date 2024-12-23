use std::str::FromStr;

use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, Either, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

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

pub fn scope() -> Scope {
    Scope::new("/19")
        .service(reset)
        .service(cite)
        .service(remove)
        .service(undo)
        .service(draft)
}
