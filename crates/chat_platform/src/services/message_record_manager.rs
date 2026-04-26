use std::sync::atomic::{AtomicI64, Ordering};

use sqlx::PgPool;

use crate::cache::RedisMessageCache;
use crate::error::{AppError, AppResult};
use crate::repositories::{
    MessageIdempotencyQuery, MessageRepository, MessageWithSenderRow, NewMessage,
};

/// 全局消息ID生成器
///
/// 在启动时从数据库加载最大 msg_id 作为初始值，
/// 之后每次调用 `next_id()` 原子递增，保证单进程内唯一。
static NEXT_MSG_ID: AtomicI64 = AtomicI64::new(0);

/// 消息记录管理器
///
/// 提供 Redis 缓存的读写分离消息管理：
/// - **写路径**：先分配ID写入 Redis 缓存，再异步落库到 PostgreSQL
/// - **读路径**：直接读取 Redis 缓存，未命中时回退到数据库并回填缓存
#[derive(Clone)]
pub struct MessageRecordManager {
    cache: RedisMessageCache,
    pool: PgPool,
}

impl MessageRecordManager {
    /// 创建管理器并初始化ID生成器
    ///
    /// 从数据库查询当前最大 `msg_id` 作为原子计数器的起始值。
    pub async fn new(pool: PgPool, cache: RedisMessageCache) -> AppResult<Self> {
        let max_id: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(msg_id), 0) FROM messages",
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        NEXT_MSG_ID.store(max_id + 1, Ordering::SeqCst);
        tracing::info!("消息ID生成器初始化完成，起始ID: {}", max_id + 1);

        Ok(Self { cache, pool })
    }

    /// 分配下一个消息ID（原子递增）
    pub fn next_id(&self) -> i64 {
        NEXT_MSG_ID.fetch_add(1, Ordering::SeqCst)
    }

    /// 发送消息：先写 Redis 缓存，再异步落库到 PostgreSQL
    ///
    /// 1. 分配 msg_id
    /// 2. 构造 MessageWithSenderRow 写入 Redis 缓存
    /// 3. spawn 异步任务将消息持久化到 PostgreSQL
    #[tracing::instrument(skip(self, new_message))]
    pub async fn send_message(&self, new_message: &NewMessage<'_>) -> AppResult<MessageWithSenderRow> {
        let msg = MessageWithSenderRow {
            msg_id: new_message.msg_id,
            group_id: new_message.group_id.to_string(),
            sender_id: new_message.sender_id.to_string(),
            sender_name: Some(new_message.sender_name.to_string()),
            sender_avatar_url: new_message.sender_avatar_url.map(|s| s.to_string()),
            content: new_message.content.to_string(),
            msg_type: new_message.msg_type.to_string(),
            idempotency_key: Some(new_message.idempotency_key.to_string()),
            created_at: new_message.created_at.to_string(),
        };

        // 先写 Redis 缓存
        self.cache.insert(&msg).await?;
        tracing::debug!(
            "消息已写入 Redis 缓存: msg_id={}, group_id={}",
            msg.msg_id,
            msg.group_id
        );

        // 异步落库到 PostgreSQL
        let pool = self.pool.clone();
        let nm = OwnedNewMessage {
            msg_id: new_message.msg_id,
            group_id: new_message.group_id.to_string(),
            sender_id: new_message.sender_id.to_string(),
            content: new_message.content.to_string(),
            msg_type: new_message.msg_type.to_string(),
            idempotency_key: new_message.idempotency_key.to_string(),
            created_at: new_message.created_at.to_string(),
            sender_name: new_message.sender_name.to_string(),
            sender_avatar_url: new_message.sender_avatar_url.map(|s| s.to_string()),
        };

        tokio::spawn(async move {
            let result = MessageRepository::insert_message_returning(
                &pool,
                &NewMessage {
                    msg_id: nm.msg_id,
                    group_id: &nm.group_id,
                    sender_id: &nm.sender_id,
                    content: &nm.content,
                    msg_type: &nm.msg_type,
                    idempotency_key: &nm.idempotency_key,
                    created_at: &nm.created_at,
                    sender_name: &nm.sender_name,
                    sender_avatar_url: nm.sender_avatar_url.as_deref(),
                },
            )
            .await;

            match result {
                Ok(row) => {
                    tracing::debug!("消息异步落库成功: msg_id={}", row.msg_id);
                }
                Err(e) => {
                    tracing::error!("消息异步落库失败: msg_id={}, error={}", nm.msg_id, e);
                }
            }
        });

        Ok(msg)
    }

    /// 幂等性检查：先查 Redis 缓存，未命中再查 PostgreSQL
    #[tracing::instrument(skip(self))]
    pub async fn find_idempotent_message(
        &self,
        query: &MessageIdempotencyQuery<'_>,
    ) -> AppResult<Option<MessageWithSenderRow>> {
        // 先查 Redis 缓存
        if let Some(msg) = self
            .cache
            .get_idempotent(query.sender_bot_id, query.group_id, query.idempotency_key)
            .await?
        {
            tracing::debug!("幂等检查命中 Redis 缓存");
            return Ok(Some(msg));
        }

        // 缓存未命中，查 PostgreSQL
        let result = MessageRepository::find_idempotent_message(&self.pool, query).await?;

        // 回填 Redis 缓存
        if let Some(ref msg) = result {
            let _ = self.cache.insert(msg).await;
        }

        Ok(result)
    }

    /// 按ID查询消息：先查 Redis 缓存，未命中再查 PostgreSQL
    pub async fn get_by_id(&self, msg_id: i64) -> AppResult<Option<MessageWithSenderRow>> {
        if let Some(msg) = self.cache.get_by_id(msg_id).await? {
            return Ok(Some(msg));
        }

        let result = MessageRepository::find_by_id_enriched(&self.pool, msg_id).await?;

        if let Some(ref msg) = result {
            let _ = self.cache.insert(msg).await;
        }

        Ok(result)
    }

    /// 获取群聊消息（游标分页）
    ///
    /// 如果 Redis 中没有该群聊的缓存数据，先从 PostgreSQL 预加载。
    pub async fn get_group_messages(
        &self,
        group_id: &str,
        base_id: i64,
        limit: i64,
    ) -> AppResult<Vec<MessageWithSenderRow>> {
        // 懒加载：首次查询该群聊时从 PostgreSQL 预加载到 Redis
        if !self.cache.has_group(group_id).await? {
            self.preload_group(group_id).await?;
        }

        self.cache
            .get_by_group_before(group_id, base_id, limit)
            .await
    }

    /// 从 PostgreSQL 预加载群聊消息到 Redis 缓存
    async fn preload_group(&self, group_id: &str) -> AppResult<()> {
        let msgs = MessageRepository::list_all_enriched_by_group(&self.pool, group_id).await?;
        tracing::info!(
            "预加载群聊消息到 Redis: group_id={}, count={}",
            group_id,
            msgs.len()
        );
        self.cache.preload_group(&msgs).await
    }

    /// 删除指定群聊的缓存消息（群聊删除时调用）
    pub async fn remove_group_messages(&self, group_id: &str) -> AppResult<()> {
        self.cache.remove_group(group_id).await
    }
}

/// 拥有所有权的 NewMessage，用于跨线程传递
struct OwnedNewMessage {
    msg_id: i64,
    group_id: String,
    sender_id: String,
    content: String,
    msg_type: String,
    idempotency_key: String,
    created_at: String,
    sender_name: String,
    sender_avatar_url: Option<String>,
}
