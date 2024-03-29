use secrecy::ExposeSecret;

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialise our configuration reader
    let mut settings = config::Config::default();
    // Add configuration values from a file named `configuration`. // It will look for any top-level file with an extension
    // that `config` knows how to parse: yaml, json, etc.
    settings.merge(config::File::with_name("configuration"))?;
    settings.try_into()
}

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: secrecy::Secret<String>,
    pub password: secrecy::Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username.expose_secret(),
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )
    }

    pub fn conn_string_no_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username.expose_secret(),
            self.password.expose_secret(),
            self.host,
            self.port
        )
    }
}
