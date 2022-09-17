use actix_web::{web, HttpRequest, HttpResponse, Responder};
use sqlx::types::Uuid;
use sqlx::PgPool;
use time::OffsetDateTime;
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

#[tracing::instrument(name = "Adding a new sub."
                    ,skip(form,conn)
                    ,fields(subscriber_email = %form.email,subscriber_name= %form.name))]
pub async fn subscribe(form: web::Form<SubscribeForm>, conn: web::Data<PgPool>) -> impl Responder {
    match persist_sub(form, conn).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(name = "Persist New Sub", skip(form, conn))]
async fn persist_sub(
    form: web::Form<SubscribeForm>,
    conn: web::Data<PgPool>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::from_bytes(*u::new_v4().as_bytes()),
        form.email,
        form.name,
        OffsetDateTime::now_utc()
    )
    .execute(conn.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
