//! WorkBuddy AI（国际版）账号管理命令。
//!
//! 全部逻辑复用 `workbuddy_account` / `workbuddy_oauth` 的 `*_for(variant)` 入口，
//! 只把变体固定为 [`WORKBUDDY_AI`]。范围是「账号管理 + 切号 + 额度」最小闭环：
//! 不包含多开实例、切号共享本地会话、自动签到和内置浏览器。

use std::time::Instant;

use tauri::{AppHandle, Emitter};

use crate::models::workbuddy::{WorkbuddyAccount, WorkbuddyOAuthStartResponse};
use crate::modules::workbuddy_ai_launch;
use crate::modules::workbuddy_variant::WORKBUDDY_AI;
use crate::modules::{logger, workbuddy_account, workbuddy_oauth};

const LOG_PREFIX: &str = WORKBUDDY_AI.log_label;
/// 登录后刷新一次账号信息，失败时保留原有账号数据。
async fn refresh_after_login(account: WorkbuddyAccount) -> WorkbuddyAccount {
    let account_id = account.id.clone();
    match workbuddy_account::refresh_account_token_for(&WORKBUDDY_AI, &account_id).await {
        Ok(refreshed) => refreshed,
        Err(error) => {
            logger::log_warn(&format!(
                "[{} OAuth] 登录后刷新失败，保留原账号信息：account_id={}, error={}",
                LOG_PREFIX, account_id, error
            ));
            account
        }
    }
}

#[tauri::command]
pub fn list_workbuddy_ai_accounts() -> Result<Vec<WorkbuddyAccount>, String> {
    workbuddy_account::list_accounts_checked_for(&WORKBUDDY_AI)
}

#[tauri::command]
pub fn delete_workbuddy_ai_account(account_id: String) -> Result<(), String> {
    workbuddy_account::remove_account_for(&WORKBUDDY_AI, &account_id)
}

#[tauri::command]
pub fn delete_workbuddy_ai_accounts(account_ids: Vec<String>) -> Result<(), String> {
    workbuddy_account::remove_accounts_for(&WORKBUDDY_AI, &account_ids)
}

#[tauri::command]
pub fn import_workbuddy_ai_from_json(json_content: String) -> Result<Vec<WorkbuddyAccount>, String> {
    workbuddy_account::import_from_json_for(&WORKBUDDY_AI, &json_content)
}

#[tauri::command]
pub async fn import_workbuddy_ai_from_local() -> Result<Vec<WorkbuddyAccount>, String> {
    let local_payload = tauri::async_runtime::spawn_blocking(|| {
        workbuddy_account::import_payload_from_local_prepared_for(&WORKBUDDY_AI)
    })
    .await
    .map_err(|error| format!("{} 本机导入后台任务失败: {}", LOG_PREFIX, error))??;

    let mut local_payload = match local_payload {
        Some(payload) => payload,
        None => {
            return Err(format!(
                "未在本机 {} 客户端中找到登录信息（凭据文件 {}）",
                LOG_PREFIX, WORKBUDDY_AI.auth_file_name
            ))
        }
    };

    match workbuddy_oauth::build_payload_from_token_for(&WORKBUDDY_AI, &local_payload.access_token)
        .await
    {
        Ok(mut payload) => {
            if payload.uid.is_none() {
                payload.uid = local_payload.uid.clone();
            }
            if payload.nickname.is_none() {
                payload.nickname = local_payload.nickname.clone();
            }
            if payload.refresh_token.is_none() {
                payload.refresh_token = local_payload.refresh_token.clone();
            }
            if payload.domain.is_none() {
                payload.domain = local_payload.domain.clone();
            }
            if payload.token_type.is_none() {
                payload.token_type = local_payload.token_type.clone();
            }
            if payload.expires_at.is_none() {
                payload.expires_at = local_payload.expires_at;
            }
            if payload.auth_raw.is_none() {
                payload.auth_raw = local_payload.auth_raw.clone();
            }
            if payload.profile_raw.is_none() {
                payload.profile_raw = local_payload.profile_raw.clone();
            }
            if payload.email.trim().is_empty() || payload.email == "unknown" {
                payload.email = local_payload.email.clone();
            }
            local_payload = payload;
        }
        Err(error) => {
            logger::log_warn(&format!(
                "[{} Import Local] 拉取账号资料失败，将保留本地导入结果：{}",
                LOG_PREFIX, error
            ));
        }
    }

    let mut account = workbuddy_account::upsert_account_for(&WORKBUDDY_AI, local_payload.clone())?;

    // 清理同一 token 下的占位账号，避免重复条目。
    for existing in workbuddy_account::list_accounts_for(&WORKBUDDY_AI) {
        if existing.id == account.id || existing.access_token != account.access_token {
            continue;
        }
        let is_placeholder = existing.email.trim().eq_ignore_ascii_case("unknown")
            || existing.email.trim().is_empty()
            || existing
                .uid
                .as_deref()
                .map(|value| value.trim().is_empty())
                .unwrap_or(true);
        if is_placeholder {
            if let Err(error) =
                workbuddy_account::remove_account_for(&WORKBUDDY_AI, &existing.id)
            {
                logger::log_warn(&format!(
                    "[{} Import Local] 清理占位账号失败：id={}, error={}",
                    LOG_PREFIX, existing.id, error
                ));
            }
        }
    }

    account = refresh_after_login(account).await;
    Ok(vec![account])
}

#[tauri::command]
pub fn export_workbuddy_ai_accounts(account_ids: Vec<String>) -> Result<String, String> {
    workbuddy_account::export_accounts_for(&WORKBUDDY_AI, &account_ids)
}

#[tauri::command]
pub async fn refresh_workbuddy_ai_token(account_id: String) -> Result<WorkbuddyAccount, String> {
    let started_at = Instant::now();
    logger::log_info(&format!(
        "[{} Command] 手动刷新账号开始：account_id={}",
        LOG_PREFIX, account_id
    ));
    let result = workbuddy_account::refresh_account_token_for(&WORKBUDDY_AI, &account_id).await;
    match &result {
        Ok(account) => logger::log_info(&format!(
            "[{} Command] 手动刷新账号完成：account_id={}, email={}, elapsed={}ms",
            LOG_PREFIX,
            account.id,
            account.email,
            started_at.elapsed().as_millis()
        )),
        Err(error) => logger::log_warn(&format!(
            "[{} Command] 手动刷新账号失败：account_id={}, elapsed={}ms, error={}",
            LOG_PREFIX,
            account_id,
            started_at.elapsed().as_millis(),
            error
        )),
    }
    result
}

#[tauri::command]
pub async fn refresh_all_workbuddy_ai_tokens() -> Result<i32, String> {
    let started_at = Instant::now();
    logger::log_info(&format!("[{} Command] 手动批量刷新开始", LOG_PREFIX));

    let results = workbuddy_account::refresh_all_tokens_for(&WORKBUDDY_AI).await?;
    let success_count = results.iter().filter(|(_, item)| item.is_ok()).count();
    let failed_count = results.len().saturating_sub(success_count);

    logger::log_info(&format!(
        "[{} Command] 手动批量刷新完成：success={}, failed={}, elapsed={}ms",
        LOG_PREFIX,
        success_count,
        failed_count,
        started_at.elapsed().as_millis()
    ));

    Ok(success_count as i32)
}

#[tauri::command]
pub async fn workbuddy_ai_oauth_login_start() -> Result<WorkbuddyOAuthStartResponse, String> {
    logger::log_info(&format!("[{} OAuth] start 命令触发", LOG_PREFIX));
    workbuddy_oauth::start_login_for(&WORKBUDDY_AI).await
}

#[tauri::command]
pub async fn workbuddy_ai_oauth_login_complete(
    login_id: String,
) -> Result<WorkbuddyAccount, String> {
    logger::log_info(&format!(
        "[{} OAuth] complete 命令触发：login_id={}",
        LOG_PREFIX, login_id
    ));

    let result: Result<WorkbuddyAccount, String> = async {
        let payload = workbuddy_oauth::complete_login_for(&WORKBUDDY_AI, &login_id).await?;
        let account = workbuddy_account::upsert_account_for(&WORKBUDDY_AI, payload)?;
        Ok(refresh_after_login(account).await)
    }
    .await;

    if let Err(error) = workbuddy_oauth::clear_pending_oauth_login_for(&WORKBUDDY_AI, &login_id) {
        logger::log_warn(&format!(
            "[{} OAuth] 清理待处理登录状态失败：login_id={}, error={}",
            LOG_PREFIX, login_id, error
        ));
    }

    let account = result?;
    logger::log_info(&format!(
        "[{} OAuth] complete 成功：account_id={}, email={}",
        LOG_PREFIX, account.id, account.email
    ));
    Ok(account)
}

#[tauri::command]
pub fn workbuddy_ai_oauth_login_cancel(login_id: Option<String>) -> Result<(), String> {
    logger::log_info(&format!(
        "[{} OAuth] cancel 命令触发：login_id={}",
        LOG_PREFIX,
        login_id.as_deref().unwrap_or("<none>")
    ));
    workbuddy_oauth::cancel_login_for(&WORKBUDDY_AI, login_id.as_deref())
}

#[tauri::command]
pub async fn add_workbuddy_ai_account_with_token(
    access_token: String,
) -> Result<WorkbuddyAccount, String> {
    let payload =
        workbuddy_oauth::build_payload_from_token_for(&WORKBUDDY_AI, &access_token).await?;
    workbuddy_account::upsert_account_for(&WORKBUDDY_AI, payload)
}

#[tauri::command]
pub async fn update_workbuddy_ai_account_tags(
    account_id: String,
    tags: Vec<String>,
) -> Result<WorkbuddyAccount, String> {
    workbuddy_account::update_account_tags_for(&WORKBUDDY_AI, &account_id, tags)
}

#[tauri::command]
pub fn get_workbuddy_ai_accounts_index_path() -> Result<String, String> {
    workbuddy_account::accounts_index_path_string_for(&WORKBUDDY_AI)
}

/// 切换账号：写本机凭据文件 → 记录当前账号 → 启动官方客户端。
///
/// 与国内版的差别是不做实例管理：直接把账号写进共享凭据目录下的
/// `workbuddy-desktop-ai.info`，再拉起官方客户端读取它。
#[tauri::command]
pub async fn inject_workbuddy_ai_to_vscode(
    app: AppHandle,
    account_id: String,
) -> Result<String, String> {
    let started_at = Instant::now();
    logger::log_info(&format!(
        "[{} Switch] 开始切换账号：account_id={}",
        LOG_PREFIX, account_id
    ));

    let account = workbuddy_account::load_account_for(&WORKBUDDY_AI, &account_id)
        .ok_or_else(|| format!("{} account not found: {}", LOG_PREFIX, account_id))?;

    let account_to_write = account.clone();
    tauri::async_runtime::spawn_blocking(move || {
        workbuddy_account::write_account_to_default_client_for(&WORKBUDDY_AI, &account_to_write)
    })
    .await
    .map_err(|error| format!("{} 账号切换后台任务失败: {}", LOG_PREFIX, error))??;

    crate::modules::provider_current_state::set_current_account_id(
        WORKBUDDY_AI.id.provider_id(),
        Some(account_id.as_str()),
    )?;

    let launch_message = match workbuddy_ai_launch::launch_app() {
        Ok(message) => message,
        Err(error) => {
            logger::log_warn(&format!(
                "[{} Switch] 切号完成但启动失败：account_id={}, error={}",
                LOG_PREFIX, account_id, error
            ));
            if error.starts_with("APP_PATH_NOT_FOUND:") {
                let _ = app.emit(
                    "app:path_missing",
                    serde_json::json!({
                        "app": workbuddy_ai_launch::APP_ID,
                        "retry": { "kind": "default" },
                    }),
                );
            }
            format!("切换完成，但启动 {} 失败：{}", workbuddy_ai_launch::APP_LABEL, error)
        }
    };

    logger::log_info(&format!(
        "[{} Switch] 切号完成：account_id={}, email={}, elapsed={}ms, launch={}",
        LOG_PREFIX,
        account.id,
        account.email,
        started_at.elapsed().as_millis(),
        launch_message
    ));

    Ok(format!("切换完成：{}（{}）", account.email, launch_message))
}

/// 解析官方客户端路径，供设置页「检测路径」按钮使用。
#[tauri::command]
pub fn detect_workbuddy_ai_app_path() -> Result<String, String> {
    workbuddy_ai_launch::resolve_launch_path().map(|path| path.to_string_lossy().to_string())
}
