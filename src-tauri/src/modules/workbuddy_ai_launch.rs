//! WorkBuddy AI（国际版）官方客户端定位与启动。
//!
//! 最小闭环的切号只需要两件事：把账号写进本机凭据文件
//! `CodeBuddyExtension/Data/Public/auth/workbuddy-desktop-ai.info`，再让官方客户端
//! 重新读取它。因此这里不做多开实例，只负责「找到 WorkBuddyAI 可执行文件 → 以分离
//! 进程启动」。
//!
//! Windows 上的路径探测走仓库已有的多来源探测（注册表 App Paths / Uninstall
//! DisplayIcon / 协议注册 / 快捷方式 / PATH），因此安装到非默认盘（例如
//! `F:\software setup\WorkBuddy\WorkBuddyAI\`）也能命中；探测失败时由用户在设置里
//! 填写 `workbuddy_ai_app_path`。
//!
//! macOS / Linux 的候选路径是按国内版布局推断的，**尚未在真机验证**。

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::modules::config;
use crate::modules::logger;

/// 用于日志与错误提示的应用名。
pub const APP_LABEL: &str = "WorkBuddy AI";

/// 前端 `app:path_missing` 事件使用的应用标识，与平台 id 一致。
pub const APP_ID: &str = "workbuddy_ai";

#[cfg(target_os = "windows")]
const WINDOWS_EXE_NAMES: &[&str] = &["WorkBuddyAI.exe", "WorkBuddy AI.exe"];
#[cfg(target_os = "windows")]
const WINDOWS_DISPLAY_KEYWORDS: &[&str] = &["workbuddy ai"];

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
#[cfg(target_os = "windows")]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
#[cfg(target_os = "windows")]
const DETACHED_PROCESS: u32 = 0x0000_0008;

/// 把用户填写的路径规整成可直接启动的可执行文件路径。
fn resolve_from_custom_path(raw: &str) -> Option<PathBuf> {
    let trimmed = raw.trim().trim_matches('"').trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = PathBuf::from(trimmed);

    #[cfg(target_os = "macos")]
    {
        // 允许用户直接填 .app 包目录。
        if path.extension().map(|ext| ext == "app").unwrap_or(false) {
            return resolve_macos_app_binary(&path);
        }
    }

    if path.is_file() {
        return Some(path);
    }

    // 允许填安装目录。
    if path.is_dir() {
        for name in platform_exe_names() {
            let candidate = path.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn platform_exe_names() -> &'static [&'static str] {
    WINDOWS_EXE_NAMES
}

#[cfg(target_os = "macos")]
fn platform_exe_names() -> &'static [&'static str] {
    &["WorkBuddy AI", "WorkBuddyAI", "Electron"]
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn platform_exe_names() -> &'static [&'static str] {
    &["workbuddy-ai", "workbuddyai", "WorkBuddyAI"]
}

/// 从 `.app` 包中取出真正的可执行文件。
#[cfg(target_os = "macos")]
fn resolve_macos_app_binary(app_dir: &Path) -> Option<PathBuf> {
    let macos_dir = app_dir.join("Contents").join("MacOS");
    if !macos_dir.is_dir() {
        return None;
    }
    for name in platform_exe_names() {
        let candidate = macos_dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    // 回退：取 MacOS 目录下第一个可执行文件。
    std::fs::read_dir(&macos_dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.is_file())
}

fn detect_from_common_paths() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let mut roots: Vec<PathBuf> = Vec::new();
        for key in ["LOCALAPPDATA", "PROGRAMFILES", "PROGRAMFILES(X86)"] {
            if let Ok(value) = std::env::var(key) {
                let base = PathBuf::from(value);
                roots.push(base.join("Programs").join("WorkBuddyAI"));
                roots.push(base.join("WorkBuddyAI"));
                roots.push(base.join("WorkBuddy AI"));
            }
        }
        for root in roots {
            for name in platform_exe_names() {
                let candidate = root.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        // 覆盖非默认安装盘：交给仓库已有的多来源探测。
        if let Some(path) = crate::modules::process::detect_windows_exec_path_by_signatures(
            APP_LABEL,
            WINDOWS_EXE_NAMES,
            &[],
            &[],
            WINDOWS_DISPLAY_KEYWORDS,
        ) {
            return Some(path);
        }
        return None;
    }

    #[cfg(target_os = "macos")]
    {
        let candidates = [
            "/Applications/WorkBuddy AI.app",
            "/Applications/WorkBuddyAI.app",
        ];
        for candidate in candidates {
            let app_dir = PathBuf::from(candidate);
            if app_dir.is_dir() {
                if let Some(binary) = resolve_macos_app_binary(&app_dir) {
                    return Some(binary);
                }
            }
        }
        return None;
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let candidates = [
            "/usr/bin/workbuddy-ai",
            "/usr/local/bin/workbuddy-ai",
            "/opt/workbuddy-ai/workbuddy-ai",
            "/opt/WorkBuddyAI/workbuddyai",
        ];
        candidates
            .into_iter()
            .map(PathBuf::from)
            .find(|path| path.is_file())
    }
}

/// 解析官方客户端可执行文件路径。
///
/// 顺序：用户配置 → 常见安装位置 → 平台自动探测。
pub fn resolve_launch_path() -> Result<PathBuf, String> {
    let configured = config::get_user_config().workbuddy_ai_app_path;
    if let Some(path) = resolve_from_custom_path(&configured) {
        return Ok(path);
    }
    if !configured.trim().is_empty() {
        logger::log_warn(&format!(
            "[{} Launch] 配置的启动路径不可用，回退自动探测: path={}",
            APP_LABEL,
            configured.trim()
        ));
    }
    if let Some(path) = detect_from_common_paths() {
        logger::log_info(&format!(
            "[{} Launch] 自动探测到客户端: {}",
            APP_LABEL,
            path.to_string_lossy()
        ));
        return Ok(path);
    }
    Err(format!(
        "APP_PATH_NOT_FOUND: 未找到 {} 客户端，请在设置中填写启动路径",
        APP_LABEL
    ))
}

/// 官方客户端是否正在运行。
///
/// 运行中时切换账号不会立刻生效（Electron 只在启动时读取凭据文件），
/// 因此调用方需要提示用户重启而不是静默失败。
#[cfg(target_os = "windows")]
pub fn is_app_running() -> bool {
    crate::modules::process::scan_app_launch_targets(APP_ID, None)
        .map(|candidates| !candidates.is_empty())
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
pub fn is_app_running() -> bool {
    false
}

fn spawn_detached(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // .app 包必须走 LaunchServices，直接 exec 包内二进制会丢 bundle 身份。
        if let Some(app_dir) = path
            .ancestors()
            .find(|ancestor| ancestor.extension().map(|ext| ext == "app").unwrap_or(false))
        {
            let mut cmd = Command::new("open");
            cmd.arg("-n").arg("-a").arg(app_dir);
            crate::modules::process::apply_managed_proxy_env_to_command(&mut cmd);
            return cmd
                .spawn()
                .map(|_| ())
                .map_err(|err| format!("启动 {} 失败：{}", APP_LABEL, err));
        }
    }

    let mut cmd = Command::new(path);
    crate::modules::process::apply_managed_proxy_env_to_command(&mut cmd);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
    }

    cmd.spawn()
        .map(|_| ())
        .map_err(|err| format!("启动 {} 失败：{}", APP_LABEL, err))
}

/// 启动官方客户端（若已在运行则不重复拉起）。
///
/// 返回给前端的提示语。
pub fn launch_app() -> Result<String, String> {
    let path = resolve_launch_path()?;
    if is_app_running() {
        logger::log_info(&format!(
            "[{} Launch] 客户端已在运行，跳过启动：{}",
            APP_LABEL,
            path.to_string_lossy()
        ));
        return Ok(format!(
            "账号已切换，但 {} 正在运行，请重启客户端后生效",
            APP_LABEL
        ));
    }
    spawn_detached(&path)?;
    logger::log_info(&format!(
        "[{} Launch] 已启动客户端：{}",
        APP_LABEL,
        path.to_string_lossy()
    ));
    Ok(format!("{} 已启动", APP_LABEL))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_custom_path_is_ignored() {
        assert!(resolve_from_custom_path("").is_none());
        assert!(resolve_from_custom_path("   ").is_none());
        assert!(resolve_from_custom_path("\"\"").is_none());
    }

    #[test]
    fn custom_path_accepts_quoted_file() {
        let mut path = std::env::temp_dir();
        path.push("workbuddy-ai-launch-test.exe");
        std::fs::write(&path, b"stub").expect("write stub");
        let quoted = format!("\"{}\"", path.to_string_lossy());
        assert_eq!(resolve_from_custom_path(&quoted), Some(path.clone()));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_path_reports_app_path_not_found() {
        let err = resolve_from_custom_path("/definitely/not/here/WorkBuddyAI.exe");
        assert!(err.is_none());
    }
}
