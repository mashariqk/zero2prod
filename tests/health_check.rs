use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;

use zero2prod::configuration::get_configuration;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub db_name: String,
}

#[tokio::test]
async fn health_check_test() {
    let test_app = spawn_app().await;
    let uri = test_app.address;
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", uri))
        .send()
        .await
        .expect("failed");
    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

#[tokio::test]
async fn subscribe_success_test() {
    let test_app = spawn_app().await;
    let uri = test_app.address;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &uri))
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await;
    assert_eq!(response.unwrap().status().as_u16(), 200);

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("No Sub");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    tear_down_db(&test_app.db_name).await;
}

#[tokio::test]
async fn subscribe_error_missing_data() {
    let test_app = spawn_app().await;
    let uri = test_app.address;
    let client = reqwest::Client::new();
    let bodies_and_errors = vec![
        ("name=le%20guin", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing all"),
    ];
    for (body, err) in bodies_and_errors {
        let response = client
            .post(format!("{}/subscriptions", &uri))
            .body(body)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("fail");
        let http_code = response.status().as_u16();
        assert_eq!(
            400, http_code,
            "HTTP code was {} instead of 400 for the body with message {}",
            http_code, err
        );
    }
}

async fn spawn_app() -> TestApp {
    let db_name = uuid::Uuid::new_v4().to_string();
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind");
    let port = listener.local_addr().unwrap().port();
    let db_pool = config_test_db(&db_name).await;
    let s = zero2prod::startup::run(listener, db_pool.clone()).expect("Failure");
    let _ = tokio::spawn(s);
    TestApp {
        address: format!("http://127.0.0.1:{}", &port),
        db_pool,
        db_name,
    }
}

async fn config_test_db(db_name: &String) -> PgPool {
    let mut config = get_configuration().unwrap();
    config.database.database_name = db_name.clone();
    let mut conn = PgConnection::connect(&config.database.conn_string_no_db())
        .await
        .expect("Failure to acquire dbless conn");
    conn.execute(format!(r#"CREATE DATABASE "{}";"#, &config.database.database_name).as_str())
        .await
        .expect("Unable to create db");
    let conn_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Cannot connect to test db instance");
    sqlx::migrate!("./migrations")
        .run(&conn_pool)
        .await
        .expect("failed to migrate test db");
    conn_pool
}

async fn tear_down_db(db_name: &String) {
    let config = get_configuration().unwrap();
    let mut conn = PgConnection::connect(&config.database.conn_string_no_db())
        .await
        .expect("Failure to acquire dbless conn");
    conn.execute(format!(r#"DROP DATABASE "{}" WITH (FORCE);"#, db_name).as_str())
        .await
        .expect("Unable to drop db");
}
