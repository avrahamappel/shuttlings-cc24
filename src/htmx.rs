use std::fmt::Display;

use actix_web::http::StatusCode;
use actix_web::web::Path;
use actix_web::{get, Either, Responder, Scope};

#[get("/star")]
async fn star() -> impl Responder {
    r#"<div id="star" class="lit"></div"#
}

#[derive(Clone, Copy)]
enum Color {
    Red,
    Blue,
    Purple,
}

impl Color {
    fn from(str: &str) -> Option<Self> {
        match str {
            "red" => Some(Self::Red),
            "blue" => Some(Self::Blue),
            "purple" => Some(Self::Purple),
            _ => None,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Red => Self::Blue,
            Self::Blue => Self::Purple,
            Self::Purple => Self::Red,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let str = match self {
            Self::Red => "red",
            Self::Blue => "blue",
            Self::Purple => "purple",
        };
        write!(f, "{str}")
    }
}

#[get("/present/{color}")]
async fn present(color_str: Path<String>) -> impl Responder {
    let Some(color) = Color::from(&color_str) else {
        return Either::Left(("", StatusCode::IM_A_TEAPOT));
    };
    Either::Right(format!(
        r#"<div class="present {color}" hx-get="/23/present/{}" hx-swap="outerHTML">
    <div class="ribbon"></div>
    <div class="ribbon"></div>
    <div class="ribbon"></div>
    <div class="ribbon"></div>
</div>"#,
        color.next()
    ))
}

#[get("/ornament/{state}/{n}")]
async fn ornament(params: Path<(String, String)>) -> impl Responder {
    let (state, n) = params.into_inner();
    let n = n
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let (class, next_state) = match state.as_str() {
        "on" => (" on", "off"),
        "off" => ("", "on"),
        _ => {
            return Either::Left(("", StatusCode::IM_A_TEAPOT));
        }
    };
    Either::Right(format!(
        r#"<div
    class="ornament{class}"
    id="ornament{n}"
    hx-trigger="load delay:2s once"
    hx-get="/23/ornament/{next_state}/{n}"
    hx-swap="outerHTML"
></div>"#
    ))
}

pub fn scope() -> Scope {
    Scope::new("/23")
        .service(star)
        .service(present)
        .service(ornament)
}