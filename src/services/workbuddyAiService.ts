import { invoke } from '@tauri-apps/api/core';
import type { WorkbuddyAccount } from '../types/workbuddy';

/**
 * WorkBuddy AI（国际版）账号服务。
 *
 * 与国内版共用同一套账号数据结构，只是后端走 `workbuddy_ai_*` 命令，
 * 凭据文件为 `workbuddy-desktop-ai.info`、网关为 `www.workbuddy.ai`。
 * 国际版没有每日签到，因此不提供 checkin 相关接口。
 */
export interface WorkbuddyAiOAuthLoginStartResponse {
  loginId: string;
  verificationUri: string;
  verificationUriComplete?: string | null;
  expiresIn: number;
  intervalSeconds: number;
}

export async function listWorkbuddyAiAccounts(): Promise<WorkbuddyAccount[]> {
  return await invoke('list_workbuddy_ai_accounts');
}

export async function deleteWorkbuddyAiAccount(accountId: string): Promise<void> {
  return await invoke('delete_workbuddy_ai_account', { accountId });
}

export async function deleteWorkbuddyAiAccounts(accountIds: string[]): Promise<void> {
  return await invoke('delete_workbuddy_ai_accounts', { accountIds });
}

export async function importWorkbuddyAiFromJson(
  jsonContent: string,
): Promise<WorkbuddyAccount[]> {
  return await invoke('import_workbuddy_ai_from_json', { jsonContent });
}

export async function importWorkbuddyAiFromLocal(): Promise<WorkbuddyAccount[]> {
  return await invoke('import_workbuddy_ai_from_local');
}

export async function exportWorkbuddyAiAccounts(accountIds: string[]): Promise<string> {
  return await invoke('export_workbuddy_ai_accounts', { accountIds });
}

export async function refreshWorkbuddyAiToken(accountId: string): Promise<WorkbuddyAccount> {
  return await invoke('refresh_workbuddy_ai_token', { accountId });
}

export async function refreshAllWorkbuddyAiTokens(): Promise<number> {
  return await invoke('refresh_all_workbuddy_ai_tokens');
}

export async function startWorkbuddyAiOAuthLogin(): Promise<WorkbuddyAiOAuthLoginStartResponse> {
  return await invoke('workbuddy_ai_oauth_login_start');
}

export async function completeWorkbuddyAiOAuthLogin(
  loginId: string,
): Promise<WorkbuddyAccount> {
  return await invoke('workbuddy_ai_oauth_login_complete', { loginId });
}

export async function cancelWorkbuddyAiOAuthLogin(loginId?: string): Promise<void> {
  return await invoke('workbuddy_ai_oauth_login_cancel', { loginId: loginId ?? null });
}

export async function addWorkbuddyAiAccountWithToken(
  accessToken: string,
): Promise<WorkbuddyAccount> {
  return await invoke('add_workbuddy_ai_account_with_token', { accessToken });
}

export async function updateWorkbuddyAiAccountTags(
  accountId: string,
  tags: string[],
): Promise<WorkbuddyAccount> {
  return await invoke('update_workbuddy_ai_account_tags', { accountId, tags });
}

export async function getWorkbuddyAiAccountsIndexPath(): Promise<string> {
  return await invoke('get_workbuddy_ai_accounts_index_path');
}

/** 切换账号：写本机凭据文件并拉起官方客户端。 */
export async function injectWorkbuddyAiToVSCode(accountId: string): Promise<string> {
  return await invoke('inject_workbuddy_ai_to_vscode', { accountId });
}

/** 自动探测官方客户端路径，供设置页使用。 */
export async function detectWorkbuddyAiAppPath(): Promise<string> {
  return await invoke('detect_workbuddy_ai_app_path');
}
