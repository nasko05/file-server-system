use once_cell::sync::Lazy;
use postgres::{Client, NoTls};
use std::sync::Mutex;

// A global static that initializes once on first use.
static GLOBAL_DB_CLIENT: Lazy<Mutex<Client>> = Lazy::new(|| {
    let host = std::env::var("POSTGRESQL_HOST").expect("POSTGRESQL_HOST must be set.");
    let user = std::env::var("POSTGRESQL_USER").expect("POSTGRESQL_USER must be set.");
    let pass = std::env::var("POSTGRESQL_PASSWORD").expect("POSTGRESQL_PASSWORD must be set.");
    let port = std::env::var("POSTGRESQL_PORT").expect("POSTGRESQL_PORT must be set.");
    let database = std::env::var("POSTGRESQL_DATABASE").expect("POSTGRESQL_DATABASE must be set.");

    let connection_str = format!(
        "host={} user={} password={} port={} dbname={}",
        host, user, pass, port, database
    );

    // Panic on error, or you could handle it differently:
    let client = Client::connect(&connection_str, NoTls)
        .expect("Failed to initialize global database client");

    Mutex::new(client)
});

/// Access the client from anywhere
pub fn get_db_client() -> std::sync::MutexGuard<'static, Client> {
    GLOBAL_DB_CLIENT.lock().expect("Failed to lock the global client")
}

pub fn verify_user_credentials(username: &str, password: &str) -> Result<(), String> {
    // Acquire the lock
    let mut client = get_db_client();

    // Use `client`...
    // e.g., do a simple query
    let rows = client
        .query("SELECT password_hash FROM users WHERE username = $1", &[&username])
        .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Err("User not found".into());
    }

    // ... more logic to verify password, etc ...
    Ok(())
}