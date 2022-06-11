use actix_web::dev::Server;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;

pub async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[derive(serde::Deserialize)]
pub struct SubscribeForm {
    email: String,
    name: String,
}

pub async fn subscribe(_form: web::Form<SubscribeForm>) -> impl Responder {
    HttpResponse::Ok()
}
