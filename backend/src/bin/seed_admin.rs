// Seed an admin user.
// Usage:
//   cargo run --bin seed_admin -- <username>            # interactive (prompts for password)
//   cargo run --bin seed_admin -- <username> <password> # non-interactive (handy in CI/scripts)
// You'll be prompted for a password if the second argument is omitted.

use std::env;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;

#[path = "../auth/password.rs"]
mod password;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let username = env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: seed_admin <username> [password]"))?;

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    let pw = if let Some(p) = env::args().nth(2) {
        p
    } else {
        let pw = rpassword::prompt_password("password: ")?;
        let pw2 = rpassword::prompt_password("confirm:  ")?;
        if pw != pw2 {
            anyhow::bail!("passwords do not match");
        }
        pw
    };
    if pw.len() < 8 {
        anyhow::bail!("password must be at least 8 characters");
    }

    let hash = password::hash(&pw)?;

    let row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO users (username, password_hash, role)
        VALUES ($1, $2, 'admin')
        ON CONFLICT (username) DO UPDATE
            SET password_hash = EXCLUDED.password_hash
        RETURNING id
        "#,
    )
    .bind(&username)
    .bind(&hash)
    .fetch_one(&pool)
    .await?;

    println!("upserted admin user '{username}' (id={})", row.0);
    Ok(())
}
