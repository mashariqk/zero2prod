use actix_web::{web, HttpRequest, HttpResponse, Responder};
use sqlx::types::Uuid;
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::Instrument;
use uuid::Uuid as u;

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

pub async fn subscribe(form: web::Form<SubscribeForm>, conn: web::Data<PgPool>) -> impl Responder {
    let req_id = u::new_v4();
    let req_span = tracing::info_span!("Adding a new sub.",%req_id,sub_email = %form.email,sub_nm = %form.name);
    let qry_span = tracing::info_span!("In the query block");
    let _ = req_span.enter();
    let uuid = Uuid::from_bytes(*u::new_v4().as_bytes());
    match sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        uuid,
        form.email,
        form.name,
        OffsetDateTime::now_utc()
    )
    .execute(conn.get_ref())
    .instrument(qry_span)
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}
