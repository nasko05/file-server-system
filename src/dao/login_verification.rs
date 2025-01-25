use bcrypt::verify;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use once_cell::sync::Lazy;
use std::env;
use tokio_postgres::{Config, NoTls};

static DB_POOL: Lazy<Pool> = Lazy::new(|| {
    let host = env::var("POSTGRESQL_HOST").expect("POSTGRESQL_HOST must be set");
    let user = env::var("POSTGRESQL_USER").expect("POSTGRESQL_USER must be set");
    let pass = env::var("POSTGRESQL_PASSWORD").expect("POSTGRESQL_PASSWORD must be set");
    let port = env::var("POSTGRESQL_PORT").expect("POSTGRESQL_PORT must be set");
    let db   = env::var("POSTGRESQL_DATABASE").expect("POSTGRESQL_DATABASE must be set");

    let mut cfg = Config::new();
    cfg.host(&host);
    cfg.user(&user);
    cfg.password(&pass);
    cfg.dbname(&db);
    cfg.port(port.parse().expect("POSTGRESQL_PORT must be a valid integer"));

    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast
    };

    let mgr = Manager::from_config(cfg, NoTls, mgr_config);

    Pool::builder(mgr)
        .max_size(16) // set max connections
        .build()
        .expect("Failed to create Deadpool Postgres pool")
});

// 2) An async function to verify user credentials
pub async fn verify_user_credentials(username: &str, password: &str) -> Result<String, String> {
    // Acquire a client from the pool (async)
    let client = DB_POOL
        .get()
        .await
        .map_err(|e| format!("Failed to get client from pool: {}", e))?;

    // Query the stored password hash
    let rows = client
        .query(
            "SELECT password_hash FROM users WHERE username = $1",
            &[&username],
        )
        .await
        .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Err("User not found".to_string());
    }

    // Extract the password hash from the first row
    let row = &rows[0];
    let hash: String = row.get("password_hash");

    // Compare the provided password with the stored hash
    let valid = verify(password, &hash).map_err(|e| e.to_string())?;

    if valid {
        Ok(username.to_string())
    } else {
        Err("Invalid credentials".to_string())
    }
}

pub async fn check_privileges(user_role: &str) -> Result<i32, String> {

    // Acquire a client from the pool (async)
    let client = DB_POOL
        .get()
        .await
        .map_err(|e| format!("Failed to get client from pool: {}", e))?;

    // Query the stored password hash
    let rows = client
        .query(
            "SELECT privilege_level FROM privilege_level WHERE role = $1",
            &[&user_role],
        )
        .await
        .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Err("User not found".to_string());
    }

    // Extract the password hash from the first row
    let row = &rows[0];
    let privilege: i16 = row.get("privilege_level");
    let privilege: i32 = privilege.into();
    Ok(privilege)
}