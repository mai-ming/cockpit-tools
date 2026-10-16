//! WorkBuddy 变体差异表（国内版 / 国际版）。
//!
//! 两个版本来自同一套官方代码：账号文件结构（`account` / `auth` / `accounts` /
//! `allAccounts`）、接口形态（`{endpoint}/v2/plugin/...`）和账号 uid 规则
//! （JWT `sub`，与 `Data/{uid}` 目录同名）完全一致，差异只在网关域名、平台标识、
//! 本机凭据文件名、配置根目录和账号存储目录。
//!
//! 本表是这些差异的唯一来源；`workbuddy_account` / `workbuddy_oauth` 通过
//! `*_for(variant)` 入口接受变体，未传变体的旧入口等价于 [`WORKBUDDY_CN`]。
//!
//! 已在本机实测确认（WorkBuddy 5.8.2 与 WorkBuddy AI 5.5.2 同时安装）：
//! - 凭据目录两版共用 `%LOCALAPPDATA%\CodeBuddyExtension\Data\Public\auth\`，
//!   仅文件名不同（`workbuddy-desktop.info` / `workbuddy-desktop-ai.info`）；
//! - 国际版 `auth.domain` 为 `www.workbuddy.ai`，Keycloak realm
//!   `https://www.workbuddy.ai/auth/realms/copilot`；
//! - 国际版官方下发的插件配置为 `applicationName=workbuddy-ai`、
//!   `platform=workbuddy-ai`、`prefixPath=/plugin`。

/// 变体标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkbuddyVariantId {
    /// 国内版 WorkBuddy。
    Cn,
    /// 国际版 WorkBuddy AI。
    Ai,
}

impl WorkbuddyVariantId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cn => "workbuddy",
            Self::Ai => "workbuddy_ai",
        }
    }

    /// 前端 provider / 平台标识，用于当前账号状态。
    pub fn provider_id(self) -> &'static str {
        self.as_str()
    }
}

/// 单个变体的全部差异项。
#[derive(Debug, Clone, Copy)]
// `config_root` / `user_data_dir_name` / `has_checkin` 供尚未实现的实例管理与签到路径
// 使用；表里先如实记录差异，避免实现时再回头核对官方布局。
#[allow(dead_code)]
pub struct WorkbuddyVariant {
    pub id: WorkbuddyVariantId,
    /// 网关根地址。
    pub api_endpoint: &'static str,
    /// 接口前缀（两版一致，取自官方 `prefixPath`）。
    pub api_prefix: &'static str,
    /// `auth/state?platform=` 的平台标识。
    pub platform: &'static str,
    /// 本机凭据文件名，位于共享 auth 目录下。
    pub auth_file_name: &'static str,
    /// cockpit 自身的账号索引文件名。
    pub accounts_index_file: &'static str,
    /// cockpit 自身的账号详情目录名。
    pub accounts_dir: &'static str,
    /// 账号详情文件加密信封里记录的 kind，仅用于诊断。
    pub storage_kind: &'static str,
    /// 官方客户端配置根目录名（位于用户主目录下）。
    pub config_root: &'static str,
    /// 官方客户端 Electron userData 目录名（位于配置根目录下）。
    pub user_data_dir_name: &'static str,
    /// 日志与错误信息前缀。
    pub log_label: &'static str,
    /// 是否支持每日签到（国际版暂无该活动）。
    pub has_checkin: bool,
}

/// 国内版 WorkBuddy。
pub const WORKBUDDY_CN: WorkbuddyVariant = WorkbuddyVariant {
    id: WorkbuddyVariantId::Cn,
    api_endpoint: "https://copilot.tencent.com",
    api_prefix: "/v2/plugin",
    platform: "workbuddy",
    auth_file_name: "workbuddy-desktop.info",
    accounts_index_file: "workbuddy_accounts.json",
    accounts_dir: "workbuddy_accounts",
    storage_kind: "workbuddy",
    config_root: ".workbuddy",
    user_data_dir_name: "app",
    log_label: "WorkBuddy",
    has_checkin: true,
};

/// 国际版 WorkBuddy AI。
pub const WORKBUDDY_AI: WorkbuddyVariant = WorkbuddyVariant {
    id: WorkbuddyVariantId::Ai,
    api_endpoint: "https://www.workbuddy.ai",
    api_prefix: "/v2/plugin",
    platform: "workbuddy-ai",
    auth_file_name: "workbuddy-desktop-ai.info",
    accounts_index_file: "workbuddy_ai_accounts.json",
    accounts_dir: "workbuddy_ai_accounts",
    storage_kind: "workbuddy_ai",
    config_root: ".workbuddy-ai",
    user_data_dir_name: "app",
    log_label: "WorkBuddy AI",
    has_checkin: false,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_keep_distinct_identity_and_storage() {
        assert_ne!(WORKBUDDY_CN.api_endpoint, WORKBUDDY_AI.api_endpoint);
        assert_ne!(WORKBUDDY_CN.platform, WORKBUDDY_AI.platform);
        assert_ne!(WORKBUDDY_CN.auth_file_name, WORKBUDDY_AI.auth_file_name);
        assert_ne!(
            WORKBUDDY_CN.accounts_index_file,
            WORKBUDDY_AI.accounts_index_file
        );
        assert_ne!(WORKBUDDY_CN.accounts_dir, WORKBUDDY_AI.accounts_dir);
        assert_ne!(WORKBUDDY_CN.config_root, WORKBUDDY_AI.config_root);
        // 共享同一套接口形态。
        assert_eq!(WORKBUDDY_CN.api_prefix, WORKBUDDY_AI.api_prefix);
    }

    #[test]
    fn international_variant_has_no_checkin_activity() {
        assert!(WORKBUDDY_CN.has_checkin);
        assert!(!WORKBUDDY_AI.has_checkin);
    }

    #[test]
    fn provider_ids_are_stable() {
        assert_eq!(WORKBUDDY_CN.id.as_str(), "workbuddy");
        assert_eq!(WORKBUDDY_AI.id.as_str(), "workbuddy_ai");
        assert_eq!(WORKBUDDY_AI.platform, "workbuddy-ai");
    }
}
