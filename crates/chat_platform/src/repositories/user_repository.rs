use crate::error::{AppError, AppResult};
use crate::models::User;
use sqlx::PgPool;

pub struct NewUser<'a> {
    pub user_id: &'a str,
    pub name: &'a str,
    pub account: &'a str,
    pub password_hash: &'a str,
    pub id_number: &'a str,
    pub avatar_url: Option<&'a str>,
    pub now: &'a str,
}

#[derive(sqlx::FromRow)]
pub struct UserLoginRow {
    pub user_id: String,
    pub name: String,
    pub password_hash: Option<String>,
}

pub struct UserRepository;

impl UserRepository {
    pub async fn insert_user(pool: &PgPool, new_user: &NewUser<'_>) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO users (user_id, name, account, password_hash, id_number, avatar_url, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(new_user.user_id)
        .bind(new_user.name)
        .bind(new_user.account)
        .bind(new_user.password_hash)
        .bind(new_user.id_number)
        .bind(new_user.avatar_url)
        .bind(new_user.now)
        .bind(new_user.now)
        .execute(pool)
        .await
        .map_err(|e| {
            let err_msg = e.to_string();
            if err_msg.contains("users_account_unique")
                || err_msg.contains("users_account_key")
                || err_msg.contains("users.account")
            {
                AppError::AccountConflict
            } else {
                AppError::DatabaseError(err_msg)
            }
        })?;

        Ok(())
    }

    pub async fn find_by_id(pool: &PgPool, user_id: &str) -> AppResult<Option<User>> {
        sqlx::query_as(
            "SELECT user_id, name, id_number, avatar_url, created_at, updated_at FROM users WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn find_login_by_account(
        pool: &PgPool,
        account: &str,
    ) -> AppResult<Option<UserLoginRow>> {
        sqlx::query_as::<_, UserLoginRow>(
            "SELECT user_id, name, password_hash FROM users WHERE account = $1",
        )
        .bind(account)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn update_profile(
        pool: &PgPool,
        user_id: &str,
        name: &str,
        avatar_url: Option<&str>,
        password_hash: Option<&str>,
        updated_at: &str,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET name = $1, avatar_url = $2, password_hash = COALESCE($3, password_hash), updated_at = $4
            WHERE user_id = $5
            "#,
        )
        .bind(name)
        .bind(avatar_url)
        .bind(password_hash)
        .bind(updated_at)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_bots_by_owner(pool: &PgPool, owner_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM bots WHERE owner_id = $1")
            .bind(owner_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_user_by_id(pool: &PgPool, user_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
