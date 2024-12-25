use std::fmt::Display;
use std::io::Read;

use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;
use actix_web::http::StatusCode;
use actix_web::web::Path;
use actix_web::{get, post, Either, Responder, Scope};
use serde::Deserialize;

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

#[derive(Debug, Deserialize)]
struct Package {
    checksum: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Lockfile {
    package: Vec<Package>,
}

#[derive(Debug, MultipartForm)]
struct LockfileForm {
    #[multipart(limit = "2MB")]
    lockfile: TempFile,
}

#[post("/lockfile")]
async fn lockfile(MultipartForm(form): MultipartForm<LockfileForm>) -> impl Responder {
    let mut file_contents = String::new();
    if form
        .lockfile
        .file
        .as_file()
        .read_to_string(&mut file_contents)
        .is_ok()
    {
        if let Ok(lockfile) = toml::from_str::<Lockfile>(&file_contents) {
            let mut sprinkles = vec![];
            for checksum in lockfile.package.iter().filter_map(|p| p.checksum.as_ref()) {
                if checksum.len() >= 10 {
                    let color = &checksum[0..6];
                    if u32::from_str_radix(color, 16).is_ok() {
                        if let Ok(top) = u8::from_str_radix(&checksum[6..8], 16) {
                            if let Ok(left) = u8::from_str_radix(&checksum[8..10], 16) {
                                sprinkles.push(format!(
                                    r#"<div style="background-color:#{color};top:{top}px;left:{left}px;"></div>"#
                                ));
                                continue;
                            }
                        }
                    }
                }
                return Either::Left(("", StatusCode::UNPROCESSABLE_ENTITY));
            }

            return Either::Right(sprinkles.join("\n"));
        }
    }
    Either::Left(("", StatusCode::BAD_REQUEST))
}

pub fn scope() -> Scope {
    Scope::new("/23")
        .service(star)
        .service(present)
        .service(ornament)
        .service(lockfile)
}
