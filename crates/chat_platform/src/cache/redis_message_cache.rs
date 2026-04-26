use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;

use crate::error::{AppError, AppResult};
use crate::repositories::MessageWithSenderRow;

/// Redis 消息缓存
///
/// 使用三种 Redis 数据结构实现消息索引：
/// - `msg:{id}` → JSON 消息（按ID查询）
/// - `grp:{group_id}` → ZSET（score=msg_id，按群组+游标分页）
/// - `idem:{sender_id}:{group_id}:{key}` → msg_id（幂等检查）
#[derive(Clone)]
pub struct RedisMessageCache {
    conn: MultiplexedConnection,
}

impl RedisMessageCache {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }

    /// 插入一条消息到缓存（同时更新三个索引）
    pub async fn insert(&self, msg: &MessageWithSenderRow) -> AppResult<()> {
        let json = serde_json::to_string(msg)
            .map_err(|e| AppError::InternalError(format!("序列化消息失败: {e}")))?;

        let mut conn = self.conn.clone();

        // 按ID索引
        let id_key = format!("msg:{}", msg.msg_id);
        redis::cmd("SET")
            .arg(&id_key)
            .arg(&json)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        // 按群组索引（ZSET，score = msg_id）
        let grp_key = format!("grp:{}", msg.group_id);
        redis::cmd("ZADD")
            .arg(&grp_key)
            .arg(msg.msg_id)
            .arg(&json)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        // 幂等索引：idempotency_key → msg_id
        if let Some(ref idem_key) = msg.idempotency_key {
            let idem_idx_key = idempotency_key_idx(&msg.sender_id, &msg.group_id, idem_key);
            let _: () = conn
                .set(&idem_idx_key, msg.msg_id)
                .await
                .map_err(|e| AppError::RedisError(e.to_string()))?;
        }

        Ok(())
    }

    /// 按消息ID查询
    pub async fn get_by_id(&self, msg_id: i64) -> AppResult<Option<MessageWithSenderRow>> {
        let id_key = format!("msg:{}", msg_id);
        let mut conn = self.conn.clone();
        let json: Option<String> = conn.get(&id_key).await.unwrap_or(None);

        match json {
            Some(j) => {
                let msg: MessageWithSenderRow = serde_json::from_str(&j)
                    .map_err(|e| AppError::InternalError(format!("反序列化消息失败: {e}")))?;
                Ok(Some(msg))
            }
            None => Ok(None),
        }
    }

    /// 按群组获取消息（游标分页，返回 ASC 顺序：旧→新）
    pub async fn get_by_group_before(
        &self,
        group_id: &str,
        base_id: i64,
        limit: i64,
    ) -> AppResult<Vec<MessageWithSenderRow>> {
        let grp_key = format!("grp:{}", group_id);
        let mut conn = self.conn.clone();

        // ZREVRANGEBYSCORE: score < base_id，按 score DESC，取 limit 条
        let jsons: Vec<String> = conn
            .zrevrangebyscore_limit(&grp_key, base_id - 1, 0, 0, limit as isize)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        let mut result: Vec<MessageWithSenderRow> = jsons
            .iter()
            .filter_map(|j| serde_json::from_str(j).ok())
            .collect();

        // 翻转得到 ASC（旧→新）以匹配 API 约定
        result.reverse();
        Ok(result)
    }

    /// 幂等性检查：按 (sender_id, group_id, idempotency_key) 查询
    pub async fn get_idempotent(
        &self,
        sender_bot_id: &str,
        group_id: &str,
        idempotency_key: &str,
    ) -> AppResult<Option<MessageWithSenderRow>> {
        let idem_key = idempotency_key_idx(sender_bot_id, group_id, idempotency_key);
        let mut conn = self.conn.clone();
        let msg_id: Option<i64> = conn.get(&idem_key).await.unwrap_or(None);

        match msg_id {
            Some(id) => self.get_by_id(id).await,
            None => Ok(None),
        }
    }

    /// 检查某个群聊是否已有缓存数据
    pub async fn has_group(&self, group_id: &str) -> AppResult<bool> {
        let grp_key = format!("grp:{}", group_id);
        let mut conn = self.conn.clone();
        let count: i64 = conn.zcard(&grp_key).await.unwrap_or(0);
        Ok(count > 0)
    }

    /// 批量预加载群聊消息到缓存
    pub async fn preload_group(&self, msgs: &[MessageWithSenderRow]) -> AppResult<()> {
        for msg in msgs {
            self.insert(msg).await?;
        }
        Ok(())
    }

    /// 删除指定群聊的所有缓存消息
    pub async fn remove_group(&self, group_id: &str) -> AppResult<()> {
        let mut conn = self.conn.clone();
        let grp_key = format!("grp:{}", group_id);

        // 先获取所有消息的JSON
        let jsons: Vec<String> = conn
            .zrange(&grp_key, 0, -1)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        for json in &jsons {
            if let Ok(msg) = serde_json::from_str::<MessageWithSenderRow>(json) {
                let id_key = format!("msg:{}", msg.msg_id);
                let _: () = conn.del(&id_key).await.unwrap_or(());

                if let Some(ref idem_key) = msg.idempotency_key {
                    let idem_idx_key = idempotency_key_idx(&msg.sender_id, &msg.group_id, idem_key);
                    let _: () = conn.del(&idem_idx_key).await.unwrap_or(());
                }
            }
        }

        let _: () = conn.del(&grp_key).await.unwrap_or(());
        Ok(())
    }
}

/// 幂等索引键格式
fn idempotency_key_idx(sender_bot_id: &str, group_id: &str, idempotency_key: &str) -> String {
    format!("idem:{sender_bot_id}:{group_id}:{idempotency_key}")
}
