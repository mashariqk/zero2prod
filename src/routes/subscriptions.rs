use actix_web::{web, HttpRequest, HttpResponse, Responder};

pub async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct SubscribeForm {
    email: String,
    name: String,
}

pub async fn subscribe(_form: web::Form<SubscribeForm>) -> impl Responder {
    HttpResponse::Ok()
}
