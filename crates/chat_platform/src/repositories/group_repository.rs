use crate::error::{AppError, AppResult};
use crate::models::{Group, GroupMemberResponse};
use sqlx::SqlitePool;

pub struct GroupMemberLink<'a> {
    pub group_id: &'a str,
    pub member_id: &'a str,
}

pub struct NewGroupMember<'a> {
    pub group_id: &'a str,
    pub member_id: &'a str,
    pub member_type: &'a str,
    pub joined_at: &'a str,
}

pub struct GroupMessagesQuery<'a> {
    pub group_id: &'a str,
    pub base_id: i64,
    pub limit: i64,
}

pub struct NewGroup<'a> {
    pub group_id: &'a str,
    pub group_code: Option<&'a str>,
    pub creator_id: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub status: &'a str,
    pub now: &'a str,
}

#[derive(sqlx::FromRow)]
pub struct GroupMessagesRow {
    pub msg_id: i64,
    pub group_id: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub sender_avatar_url: Option<String>,
    pub content: String,
    pub msg_type: String,
    pub created_at: String,
}

pub struct GroupRepository;

impl GroupRepository {
    pub async fn insert_group(pool: &SqlitePool, new_group: &NewGroup<'_>) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO groups (group_id, group_code, creator_id, name, description, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(new_group.group_id)
        .bind(new_group.group_code)
        .bind(new_group.creator_id)
        .bind(new_group.name)
        .bind(new_group.description)
        .bind(new_group.status)
        .bind(new_group.now)
        .bind(new_group.now)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn add_member_ignore(pool: &SqlitePool, member: &NewGroupMember<'_>) -> AppResult<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO group_members (group_id, member_id, member_type, joined_at) VALUES (?, ?, ?, ?)",
        )
        .bind(member.group_id)
        .bind(member.member_id)
        .bind(member.member_type)
        .bind(member.joined_at)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn list_all(pool: &SqlitePool) -> AppResult<Vec<Group>> {
        sqlx::query_as(
            r#"
            SELECT g.group_id, g.group_code, g.creator_id, g.name, g.description, g.status, g.created_at, g.updated_at
            FROM groups g
            ORDER BY g.created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_visible_by_user(pool: &SqlitePool, user_id: &str) -> AppResult<Vec<Group>> {
        sqlx::query_as(
            r#"
            SELECT DISTINCT g.group_id, g.group_code, g.creator_id, g.name, g.description, g.status, g.created_at, g.updated_at
            FROM groups g
            LEFT JOIN group_members gm ON gm.group_id = g.group_id
            LEFT JOIN bots b ON b.bot_id = gm.member_id
            WHERE g.creator_id = ? OR b.owner_id = ?
            ORDER BY g.created_at DESC
            "#,
        )
        .bind(user_id)
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn find_by_id(pool: &SqlitePool, group_id: &str) -> AppResult<Option<Group>> {
        sqlx::query_as(
            "SELECT group_id, group_code, creator_id, name, description, status, created_at, updated_at FROM groups WHERE group_id = ?",
        )
        .bind(group_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn find_group_id_by_code(pool: &SqlitePool, group_code: &str) -> AppResult<Option<String>> {
        sqlx::query_scalar("SELECT group_id FROM groups WHERE group_code = ?")
            .bind(group_code)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn exists(pool: &SqlitePool, group_id: &str) -> AppResult<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM groups WHERE group_id = ?)")
            .bind(group_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn is_member(pool: &SqlitePool, link: &GroupMemberLink<'_>) -> AppResult<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = ? AND member_id = ?)")
            .bind(link.group_id)
            .bind(link.member_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn remove_member(pool: &SqlitePool, link: &GroupMemberLink<'_>) -> AppResult<()> {
        sqlx::query("DELETE FROM group_members WHERE group_id = ? AND member_id = ?")
            .bind(link.group_id)
            .bind(link.member_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_messages_by_group(pool: &SqlitePool, group_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM messages WHERE group_id = ?")
            .bind(group_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_members_by_group(pool: &SqlitePool, group_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM group_members WHERE group_id = ?")
            .bind(group_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_group(pool: &SqlitePool, group_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM groups WHERE group_id = ?")
            .bind(group_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn list_messages(
        pool: &SqlitePool,
        query: &GroupMessagesQuery<'_>,
    ) -> AppResult<Vec<GroupMessagesRow>> {
        sqlx::query_as(
            r#"
            SELECT
                recent.msg_id,
                recent.group_id,
                recent.sender_id,
                b.name as sender_name,
                b.avatar_url as sender_avatar_url,
                recent.content,
                recent.msg_type,
                recent.created_at
            FROM (
                SELECT msg_id, group_id, sender_id, content, msg_type, created_at
                FROM messages
                WHERE group_id = ? AND msg_id < ?
                ORDER BY msg_id DESC
                LIMIT ?
            ) recent
            LEFT JOIN bots b ON b.bot_id = recent.sender_id
            ORDER BY recent.msg_id ASC
            "#,
        )
        .bind(query.group_id)
        .bind(query.base_id)
        .bind(query.limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn can_user_view_members(pool: &SqlitePool, group_id: &str, user_id: &str) -> AppResult<bool> {
        sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM groups g
                LEFT JOIN group_members gm ON gm.group_id = g.group_id
                LEFT JOIN bots b ON b.bot_id = gm.member_id
                WHERE g.group_id = ? AND (g.creator_id = ? OR b.owner_id = ?)
            )
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_members(pool: &SqlitePool, group_id: &str) -> AppResult<Vec<GroupMemberResponse>> {
        sqlx::query_as(
            r#"
            SELECT gm.group_id, gm.member_id, gm.member_type, gm.joined_at, b.name as bot_name, b.owner_id
            FROM group_members gm
            LEFT JOIN bots b ON b.bot_id = gm.member_id
            WHERE gm.group_id = ?
            ORDER BY gm.joined_at
            "#,
        )
        .bind(group_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_all_group_ids(pool: &SqlitePool) -> AppResult<Vec<String>> {
        sqlx::query_scalar("SELECT group_id FROM groups ORDER BY created_at ASC")
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_group_ids_by_member(pool: &SqlitePool, member_id: &str) -> AppResult<Vec<String>> {
        sqlx::query_scalar("SELECT group_id FROM group_members WHERE member_id = ? ORDER BY joined_at ASC")
            .bind(member_id)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}
