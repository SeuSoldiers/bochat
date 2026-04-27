use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FileScanRecord {
    pub file_id: String,
    pub status: String,
    pub risk_level: String,
    pub scan_result: Option<String>,
    pub scanned_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
