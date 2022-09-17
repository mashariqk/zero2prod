use std::net::TcpListener;

use sqlx::PgPool;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failure to read config");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind");
    let sub = telemetry::get_subscriber("zero2prod", "info", std::io::stdout);
    telemetry::init(sub);
    run(
        listener,
        PgPool::connect(configuration.database.connection_string().as_str())
            .await
            .expect("Conn Pool Err"),
    )?
    .await
}
