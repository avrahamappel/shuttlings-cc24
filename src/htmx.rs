use actix_web::{get, Scope};

#[get("/star")]
async fn star() -> &'static str {
    r#"<div id="star" class="lit"></div"#
}

pub fn scope() -> Scope {
    Scope::new("/23").service(star)
}
