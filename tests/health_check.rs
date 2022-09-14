use std::net::TcpListener;

use sqlx::PgPool;

use zero2prod::configuration::get_configuration;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
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
    let config = get_configuration().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind");
    let port = listener.local_addr().unwrap().port();
    let conn_pool = PgPool::connect(config.database.connection_string().as_str())
        .await
        .unwrap();
    let s = zero2prod::startup::run(listener, conn_pool.clone()).expect("Failvure");
    let _ = tokio::spawn(s);
    TestApp {
        address: format!("http://127.0.0.1:{}", &port),
        db_pool: conn_pool,
    }
}
