use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::repositories::MessageWithSenderRow;

/// 消息内存缓存
///
/// 三个索引结构，支持不同维度的快速查询：
/// - `by_id`: msg_id → 消息（O(1) 单条查询，DashMap 保证并发安全）
/// - `by_group`: group_id → 消息列表（按 msg_id DESC 排序，RwLock 保护）
/// - `by_idempotency`: (sender_id, group_id, idempotency_key) → 消息（O(1) 幂等检查）
#[derive(Clone)]
pub struct MessageCache {
    /// msg_id → 消息
    by_id: Arc<DashMap<i64, MessageWithSenderRow>>,
    /// group_id → 消息列表（按 msg_id DESC 排序，最新在前）
    by_group: Arc<RwLock<HashMap<String, Vec<MessageWithSenderRow>>>>,
    /// (sender_id, group_id, idempotency_key) → 消息
    by_idempotency: Arc<DashMap<(String, String, String), MessageWithSenderRow>>,
}

impl MessageCache {
    pub fn new() -> Self {
        Self {
            by_id: Arc::new(DashMap::new()),
            by_group: Arc::new(RwLock::new(HashMap::new())),
            by_idempotency: Arc::new(DashMap::new()),
        }
    }

    /// 插入一条消息到缓存
    /// 同时更新 by_id、by_group、by_idempotency 三个索引
    pub async fn insert(&self, msg: MessageWithSenderRow) {
        // 幂等索引
        if let Some(ref idem_key) = msg.idempotency_key {
            self.by_idempotency.insert(
                (msg.sender_id.clone(), msg.group_id.clone(), idem_key.clone()),
                msg.clone(),
            );
        }

        // 按ID索引
        self.by_id.insert(msg.msg_id, msg.clone());

        // 按群组索引（维护 msg_id DESC 顺序）
        let mut by_group = self.by_group.write().await;
        let msgs = by_group.entry(msg.group_id.clone()).or_default();
        // 二分查找插入位置以维持 DESC 顺序
        let pos = msgs.partition_point(|m| m.msg_id > msg.msg_id);
        msgs.insert(pos, msg);
    }

    /// 按消息ID查询
    pub fn get_by_id(&self, msg_id: i64) -> Option<MessageWithSenderRow> {
        self.by_id.get(&msg_id).map(|r| r.clone())
    }

    /// 按群组获取消息（游标分页）
    /// 返回 (msg_id < base_id) 的最新 limit 条消息，按 msg_id ASC 排列（旧→新）
    pub async fn get_by_group_before(
        &self,
        group_id: &str,
        base_id: i64,
        limit: i64,
    ) -> Vec<MessageWithSenderRow> {
        let by_group = self.by_group.read().await;
        let Some(msgs) = by_group.get(group_id) else {
            return Vec::new();
        };

        let mut result: Vec<_> = msgs
            .iter()
            .filter(|m| m.msg_id < base_id)
            .take(limit as usize)
            .cloned()
            .collect();
        // 缓存内部按 msg_id DESC 存储，翻转得到 ASC（旧→新）以匹配 API 约定
        result.reverse();
        result
    }

    /// 幂等性检查：按 (sender_id, group_id, idempotency_key) 查询
    pub fn get_idempotent(
        &self,
        sender_bot_id: &str,
        group_id: &str,
        idempotency_key: &str,
    ) -> Option<MessageWithSenderRow> {
        self.by_idempotency
            .get(&(
                sender_bot_id.to_string(),
                group_id.to_string(),
                idempotency_key.to_string(),
            ))
            .map(|r| r.clone())
    }

    /// 删除指定群聊的所有缓存消息
    pub async fn remove_group(&self, group_id: &str) {
        let mut by_group = self.by_group.write().await;
        if let Some(msgs) = by_group.remove(group_id) {
            for msg in &msgs {
                self.by_id.remove(&msg.msg_id);
                if let Some(ref idem_key) = msg.idempotency_key {
                    self.by_idempotency.remove(&(
                        msg.sender_id.clone(),
                        msg.group_id.clone(),
                        idem_key.clone(),
                    ));
                }
            }
        }
    }

    /// 批量预加载群聊消息到缓存（用于首次查询时的懒加载）
    pub async fn preload_group(&self, msgs: Vec<MessageWithSenderRow>) {
        for msg in msgs {
            self.insert(msg).await;
        }
    }

    /// 检查某个群聊是否已有缓存数据
    pub async fn has_group(&self, group_id: &str) -> bool {
        let by_group = self.by_group.read().await;
        by_group.contains_key(group_id)
    }
}
