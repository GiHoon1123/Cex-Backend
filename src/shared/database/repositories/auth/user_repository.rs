use sqlx::{PgPool, Row};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use crate::domains::auth::models::user::User;


pub struct UserRepository {
    pool: PgPool
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        username: Option<&str>,
    ) -> Result<User> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (email, password_hash, username, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, email, password_hash, username, created_at, updated_at
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(username)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .context("Failed to create user")?;

        Ok(User {
            id: row.get::<i64, _>("id") as u64,
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            username: row.get("username"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    // 이메일로 사용자 조회 (로그인용)
    // Get user by email (for login)
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, username, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by email")?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        Ok(Some(User {
            id: row.get::<i64, _>("id") as u64,
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            username: row.get("username"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

        // ID로 사용자 조회
    // Get user by ID
    pub async fn get_user_by_id(&self, id: u64) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, username, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by id")?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        Ok(Some(User {
            id: row.get::<i64, _>("id") as u64,
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            username: row.get("username"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }
}
