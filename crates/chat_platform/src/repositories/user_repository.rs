use crate::error::{AppError, AppResult};
use crate::models::User;
use sqlx::SqlitePool;

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
    pub async fn insert_user(pool: &SqlitePool, new_user: &NewUser<'_>) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO users (user_id, name, account, password_hash, id_number, avatar_url, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
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
            if err_msg.contains("users.account") || err_msg.contains("idx_users_account_unique") {
                AppError::AccountConflict
            } else {
                AppError::DatabaseError(err_msg)
            }
        })?;

        Ok(())
    }

    pub async fn find_by_id(pool: &SqlitePool, user_id: &str) -> AppResult<Option<User>> {
        sqlx::query_as(
            "SELECT user_id, name, id_number, avatar_url, created_at, updated_at FROM users WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn find_login_by_account(
        pool: &SqlitePool,
        account: &str,
    ) -> AppResult<Option<UserLoginRow>> {
        sqlx::query_as::<_, UserLoginRow>(
            "SELECT user_id, name, password_hash FROM users WHERE account = ?",
        )
        .bind(account)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn update_profile(
        pool: &SqlitePool,
        user_id: &str,
        name: &str,
        avatar_url: Option<&str>,
        password_hash: Option<&str>,
        updated_at: &str,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET name = ?, avatar_url = ?, password_hash = COALESCE(?, password_hash), updated_at = ?
            WHERE user_id = ?
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

    pub async fn delete_bots_by_owner(pool: &SqlitePool, owner_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM bots WHERE owner_id = ?")
            .bind(owner_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_user_by_id(pool: &SqlitePool, user_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
