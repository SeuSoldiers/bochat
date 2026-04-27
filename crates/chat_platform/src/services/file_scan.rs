use serde_json::json;

use crate::{
    db::DbPool,
    repositories::{FileScanRepository, FileScanResultUpdate, GroupRepository, NewPendingFileScan},
    services::{
        audit::{record_best_effort, AuditRecord},
        notification::{create_best_effort as create_notification_best_effort, NotificationRecord},
    },
    AppState,
};

const SCAN_WORKER_ID: &str = "file_scan_worker";

pub struct UploadScanInput<'a> {
    pub file_id: &'a str,
    pub filename: &'a str,
    pub mime_type: &'a str,
    pub size: i64,
    pub storage_path: &'a str,
    pub uploader_bot_id: &'a str,
    pub uploader_user_id: &'a str,
}

pub async fn enqueue_uploaded_file_scan_best_effort(
    state: &AppState,
    input: UploadScanInput<'_>,
) {
    if let Err(err) = enqueue_uploaded_file_scan(state, input).await {
        tracing::warn!("文件扫描任务入队失败(已忽略): {}", err);
    }
}

async fn enqueue_uploaded_file_scan(state: &AppState, input: UploadScanInput<'_>) -> crate::error::AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let inserted = FileScanRepository::insert_pending_ignore(
        &state.pool,
        &NewPendingFileScan {
            file_id: input.file_id,
            now: &now,
        },
    )
    .await?;

    if !inserted {
        return Ok(());
    }

    record_best_effort(
        &state.pool,
        AuditRecord {
            actor_type: "system",
            actor_id: SCAN_WORKER_ID,
            user_id: Some(input.uploader_user_id),
            bot_id: Some(input.uploader_bot_id),
            group_id: None,
            action: "file.scan.queued",
            resource_type: "file",
            resource_id: Some(input.file_id),
            details: Some(json!({
                "filename": input.filename,
                "mime_type": input.mime_type,
                "size": input.size
            })),
        },
    )
    .await;

    let job = ScanJob {
        file_id: input.file_id.to_string(),
        filename: input.filename.to_string(),
        mime_type: input.mime_type.to_string(),
        size: input.size,
        storage_path: input.storage_path.to_string(),
        uploader_bot_id: input.uploader_bot_id.to_string(),
        uploader_user_id: input.uploader_user_id.to_string(),
    };
    let pool = state.pool.clone();
    tokio::spawn(async move {
        process_scan_job(pool, job).await;
    });

    Ok(())
}

pub async fn notify_group_owner_if_file_flagged_best_effort(
    state: &AppState,
    msg_type: &str,
    content: &serde_json::Value,
    group_id: &str,
    sender_bot_id: &str,
    sender_user_id: &str,
) {
    if let Err(err) = notify_group_owner_if_file_flagged(
        &state.pool,
        msg_type,
        content,
        group_id,
        sender_bot_id,
        sender_user_id,
    )
    .await
    {
        tracing::warn!("发送群主文件告警失败(已忽略): {}", err);
    }
}

async fn notify_group_owner_if_file_flagged(
    pool: &DbPool,
    msg_type: &str,
    content: &serde_json::Value,
    group_id: &str,
    sender_bot_id: &str,
    sender_user_id: &str,
) -> crate::error::AppResult<()> {
    let Some(file_id) = extract_file_id_from_message(msg_type, content) else {
        return Ok(());
    };

    let Some(scan_record) = FileScanRepository::find_by_file_id(pool, &file_id).await? else {
        return Ok(());
    };
    if scan_record.status != "suspicious" {
        return Ok(());
    }

    let Some(group) = GroupRepository::find_by_id(pool, group_id).await? else {
        return Ok(());
    };

    if group.creator_id == sender_user_id {
        return Ok(());
    }

    create_notification_best_effort(
        pool,
        NotificationRecord {
            recipient_user_id: &group.creator_id,
            kind: "file_scan_alert",
            title: "群文件安全告警",
            content: &format!(
                "群 {} 检测到可疑文件（ID: {}），请尽快核查。",
                group.name, file_id
            ),
            requires_action: false,
            action_payload: Some(json!({
                "file_id": file_id,
                "group_id": group.group_id,
                "group_name": group.name,
                "sender_bot_id": sender_bot_id,
                "scan_status": scan_record.status,
                "risk_level": scan_record.risk_level,
                "scan_result": scan_record.scan_result
            })),
            related_request_id: None,
            related_group_id: Some(&group.group_id),
            related_bot_id: Some(sender_bot_id),
        },
    )
    .await;

    record_best_effort(
        pool,
        AuditRecord {
            actor_type: "system",
            actor_id: SCAN_WORKER_ID,
            user_id: Some(&group.creator_id),
            bot_id: Some(sender_bot_id),
            group_id: Some(group_id),
            action: "file.scan.group_owner_notified",
            resource_type: "file",
            resource_id: Some(&file_id),
            details: Some(json!({
                "recipient_user_id": group.creator_id,
                "scan_status": scan_record.status
            })),
        },
    )
    .await;

    Ok(())
}

#[derive(Clone, Debug)]
struct ScanJob {
    file_id: String,
    filename: String,
    mime_type: String,
    size: i64,
    storage_path: String,
    uploader_bot_id: String,
    uploader_user_id: String,
}

#[derive(Clone, Debug)]
struct ScanOutcome {
    status: &'static str,
    risk_level: &'static str,
    reasons: Vec<String>,
}

async fn process_scan_job(pool: DbPool, job: ScanJob) {
    let started_at = chrono::Utc::now().to_rfc3339();
    let (status, risk_level, reasons) = match run_simple_scan(&job).await {
        Ok(outcome) => (outcome.status, outcome.risk_level, outcome.reasons),
        Err(err) => ("failed", "high", vec![format!("扫描失败: {}", err)]),
    };

    let scan_result = json!({
        "engine": "bochat-simple-scan-v1",
        "filename": job.filename,
        "mime_type": job.mime_type,
        "size": job.size,
        "reasons": reasons
    });
    let scan_result_string = scan_result.to_string();
    let now = chrono::Utc::now().to_rfc3339();

    if let Err(err) = FileScanRepository::update_result(
        &pool,
        &FileScanResultUpdate {
            file_id: &job.file_id,
            status,
            risk_level,
            scan_result: Some(&scan_result_string),
            scanned_at: &started_at,
            now: &now,
        },
    )
    .await
    {
        tracing::error!("更新文件扫描结果失败: file_id={}, error={}", job.file_id, err);
        return;
    }

    record_best_effort(
        &pool,
        AuditRecord {
            actor_type: "system",
            actor_id: SCAN_WORKER_ID,
            user_id: Some(&job.uploader_user_id),
            bot_id: Some(&job.uploader_bot_id),
            group_id: None,
            action: "file.scan.completed",
            resource_type: "file",
            resource_id: Some(&job.file_id),
            details: Some(json!({
                "status": status,
                "risk_level": risk_level,
                "result": scan_result
            })),
        },
    )
    .await;

    if status == "suspicious" || status == "failed" {
        create_notification_best_effort(
            &pool,
            NotificationRecord {
                recipient_user_id: &job.uploader_user_id,
                kind: "file_scan_alert",
                title: "Bot 文件安全告警",
                content: &format!(
                    "Bot 上传文件 {} 检测结果为 {}，请尽快处理。",
                    job.filename, status
                ),
                requires_action: false,
                action_payload: Some(json!({
                    "file_id": job.file_id,
                    "filename": job.filename,
                    "mime_type": job.mime_type,
                    "status": status,
                    "risk_level": risk_level,
                    "scan_result": scan_result
                })),
                related_request_id: None,
                related_group_id: None,
                related_bot_id: Some(&job.uploader_bot_id),
            },
        )
        .await;

        record_best_effort(
            &pool,
            AuditRecord {
                actor_type: "system",
                actor_id: SCAN_WORKER_ID,
                user_id: Some(&job.uploader_user_id),
                bot_id: Some(&job.uploader_bot_id),
                group_id: None,
                action: "file.scan.bot_owner_notified",
                resource_type: "file",
                resource_id: Some(&job.file_id),
                details: Some(json!({
                    "status": status,
                    "risk_level": risk_level
                })),
            },
        )
        .await;
    }
}

async fn run_simple_scan(job: &ScanJob) -> Result<ScanOutcome, std::io::Error> {
    let bytes = tokio::fs::read(&job.storage_path).await?;
    let mut reasons = Vec::new();
    let mut high_risk = false;

    let filename_lower = job.filename.to_lowercase();
    if has_suspicious_extension(&filename_lower) {
        reasons.push("文件扩展名属于高风险可执行类型".to_string());
        high_risk = true;
    }
    if has_double_extension(&filename_lower) {
        reasons.push("检测到双重扩展名伪装".to_string());
        high_risk = true;
    }
    if contains_filename_risk_keyword(&filename_lower) {
        reasons.push("文件名包含高风险关键词".to_string());
    }
    if is_executable_magic(&bytes) {
        reasons.push("文件头签名疑似可执行二进制".to_string());
        high_risk = true;
    }
    if has_illegal_text_content(&job.mime_type, &bytes) {
        reasons.push("文本内容命中非法/高风险关键词".to_string());
        high_risk = true;
    }

    if bytes.len() as i64 != job.size {
        reasons.push("文件大小与记录不一致".to_string());
    }

    if reasons.is_empty() {
        return Ok(ScanOutcome {
            status: "clean",
            risk_level: "low",
            reasons,
        });
    }

    Ok(ScanOutcome {
        status: "suspicious",
        risk_level: if high_risk { "high" } else { "medium" },
        reasons,
    })
}

fn has_suspicious_extension(filename_lower: &str) -> bool {
    const RISK_EXTENSIONS: [&str; 14] = [
        ".exe", ".dll", ".bat", ".cmd", ".com", ".scr", ".msi", ".ps1", ".jar", ".sh", ".vbs",
        ".js", ".hta", ".apk",
    ];
    RISK_EXTENSIONS
        .iter()
        .any(|ext| filename_lower.ends_with(ext))
}

fn has_double_extension(filename_lower: &str) -> bool {
    const DISGUISED_SAFE_EXT: [&str; 8] = [
        ".jpg.", ".jpeg.", ".png.", ".gif.", ".pdf.", ".doc.", ".docx.", ".txt.",
    ];
    DISGUISED_SAFE_EXT.iter().any(|pattern| {
        filename_lower.contains(pattern)
            && has_suspicious_extension(filename_lower)
    })
}

fn contains_filename_risk_keyword(filename_lower: &str) -> bool {
    const KEYWORDS: [&str; 8] = [
        "keygen",
        "crack",
        "hacktool",
        "trojan",
        "ransom",
        "payload",
        "backdoor",
        "exploit",
    ];
    KEYWORDS.iter().any(|word| filename_lower.contains(word))
}

fn is_executable_magic(bytes: &[u8]) -> bool {
    (bytes.len() >= 2 && &bytes[0..2] == b"MZ")
        || (bytes.len() >= 4 && bytes[0] == 0x7f && bytes[1] == b'E' && bytes[2] == b'L' && bytes[3] == b'F')
}

fn has_illegal_text_content(mime_type: &str, bytes: &[u8]) -> bool {
    let mime = mime_type.to_ascii_lowercase();
    if !(mime.starts_with("text/")
        || mime.contains("json")
        || mime.contains("xml")
        || mime.contains("javascript")
        || mime.contains("html"))
    {
        return false;
    }

    let sample_len = bytes.len().min(64 * 1024);
    let text = String::from_utf8_lossy(&bytes[..sample_len]).to_lowercase();
    const ILLEGAL_KEYWORDS: [&str; 8] = [
        "木马",
        "勒索病毒",
        "赌博平台",
        "毒品交易",
        "枪支买卖",
        "恐怖主义",
        "malware",
        "ransomware",
    ];
    ILLEGAL_KEYWORDS.iter().any(|word| text.contains(word))
}

fn extract_file_id_from_message(msg_type: &str, content: &serde_json::Value) -> Option<String> {
    if msg_type != "file" {
        return None;
    }

    if let Some(file_id) = content
        .get("file_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return Some(file_id.to_string());
    }

    let url = content.get("url")?.as_str()?;
    extract_file_id_from_download_url(url)
}

fn extract_file_id_from_download_url(url: &str) -> Option<String> {
    const FILE_DOWNLOAD_PATH_MARKER: &str = "/api/v1/file/download/";
    let marker_pos = url.find(FILE_DOWNLOAD_PATH_MARKER)?;
    let rest = &url[marker_pos + FILE_DOWNLOAD_PATH_MARKER.len()..];
    let file_id = rest.split('/').next()?.trim();
    if file_id.is_empty() {
        return None;
    }
    Some(file_id.to_string())
}
