import { useWorkbuddyAiAccountStore } from '../stores/useWorkbuddyAiAccountStore';
import * as workbuddyAiService from '../services/workbuddyAiService';
import {
  WorkbuddyAccount,
  getWorkbuddyAccountDisplayEmail,
  getWorkbuddyPlanBadge,
  getWorkbuddyUsage,
  getWorkbuddyQuotaCategoryGroups,
} from '../types/workbuddy';
import { useProviderAccountsPage } from '../hooks/useProviderAccountsPage';
import {
  CodebuddySuiteAccountsSharedView,
  type CodebuddySuiteAccountsPlatformConfig,
} from '../components/codebuddy-suite/CodebuddySuiteAccountsSharedView';

const WORKBUDDY_AI_FLOW_NOTICE_COLLAPSED_KEY = 'agtools.workbuddy_ai.flow_notice_collapsed';
const WORKBUDDY_AI_CURRENT_ACCOUNT_ID_KEY = 'agtools.workbuddy_ai.current_account_id';

/**
 * WorkBuddy AI（国际版）账号页。
 *
 * 与国内版共用 `CodebuddySuiteAccountsSharedView`，差异集中在凭据文件
 * （`workbuddy-desktop-ai.info`）与网关（`www.workbuddy.ai`）。国际版没有每日签到、
 * 多开实例和切号共享会话，因此这里只保留账号总览页签。
 */
const workbuddyAiPlatformConfig: CodebuddySuiteAccountsPlatformConfig<WorkbuddyAccount> = {
  pageClassName: 'workbuddy-accounts-page',
  // 通用文案直接复用国内版的 workbuddy.* 键，避免 18 个语言包重复维护；
  // 只有国际版特有的说明（凭据文件与网关不同）才新增 workbuddyAi.* 键。
  searchPlaceholderKey: 'workbuddy.search',
  searchPlaceholderDefault: '搜索 WorkBuddy 账号...',
  flowNotice: {
    titleKey: 'workbuddyAi.flowNotice.title',
    titleDefault: 'WorkBuddy AI（国际版）账号管理说明（点击展开/收起）',
    descKey: 'workbuddyAi.flowNotice.desc',
    descDefault:
      '切号会把登录文件写入本机，本机导入读取同一登录文件，数据仅在本地处理；检测到无法解密的官方封装时会保留原文件并停止覆盖。',
    permissionKey: 'workbuddyAi.flowNotice.permission',
    permissionDefault:
      '权限范围：读写本机 WorkBuddy AI 登录文件（CodeBuddyExtension/Data/Public/auth/workbuddy-desktop-ai.info），用于本机导入与账号切换。',
    networkKey: 'workbuddyAi.flowNotice.network',
    networkDefault:
      '网络范围：OAuth 授权登录与 Token 刷新需联网请求 WorkBuddy AI（www.workbuddy.ai）；不上传本地登录文件或凭证原文。',
  },
  noAccountsKey: 'workbuddy.noAccounts',
  noAccountsDefault: '暂无 WorkBuddy 账号',
  addAccountTitleKey: 'workbuddy.addAccount',
  addAccountTitleDefault: '添加 WorkBuddy 账号',
  oauthDescKey: 'workbuddy.oauthDesc',
  oauthDescDefault: '点击下方按钮将在浏览器中打开 WorkBuddy 授权页面。',
  oauthFeatureCardClassName: 'workbuddy-oauth-feature-card',
  oauthFeatureTitleKey: 'workbuddy.oauthFeature.oauth.title',
  oauthFeatureTitleDefault: '仅授权登录信息',
  oauthFeatureItem1Key: 'workbuddy.oauthFeature.oauth.item1',
  oauthFeatureItem1Default: '在浏览器完成 OAuth 后即可添加账号并用于 IDE 切换。',
  oauthFeatureItem2Key: 'workbuddy.oauthFeature.oauth.item2',
  oauthFeatureItem2Default: '授权完成后会自动刷新资源包配额数据。',
  oauthFeatureItem3Key: 'workbuddy.oauthFeature.oauth.item3',
  oauthFeatureItem3Default: '账号卡片将按资源包展示额度、进度和刷新/到期时间。',
  oauthUrlInputPlaceholderKey: 'workbuddy.oauthUrlInputPlaceholder',
  oauthUrlInputPlaceholderDefault: '可手动输入授权地址',
  oauthWaitingKey: 'workbuddy.oauthWaiting',
  oauthWaitingDefault: '等待授权完成...',
  tokenDescKey: 'workbuddy.tokenDesc',
  tokenDescDefault: '粘贴 WorkBuddy 的 access token：',
  importLocalDescKey: 'workbuddy.import.localDesc',
  importLocalDescDefault: '支持从本机 WorkBuddy 客户端或 JSON 文件导入账号数据。',
  importLocalClientKey: 'workbuddyAi.import.localClient',
  importLocalClientDefault: '从本机 WorkBuddy AI 导入',
  getDisplayEmail: (account) => getWorkbuddyAccountDisplayEmail(account),
  getPlanBadge: (account) => getWorkbuddyPlanBadge(account),
  getUsage: (account) => getWorkbuddyUsage(account),
  getQuotaGroups: (account, t) => getWorkbuddyQuotaCategoryGroups(account, t),
  hasQuotaData: (_account, groups) => groups.some((group) => group.items.length > 0),
  usagePrefix: 'workbuddy',
  quotaPrefix: 'workbuddy',
  tableUsageClassName: 'workbuddy-table-usage',
};

export function WorkbuddyAiAccountsPage() {
  const store = useWorkbuddyAiAccountStore();

  const page = useProviderAccountsPage<WorkbuddyAccount>({
    platformKey: 'WorkBuddy AI',
    oauthLogPrefix: 'WorkbuddyAiOAuth',
    flowNoticeCollapsedKey: WORKBUDDY_AI_FLOW_NOTICE_COLLAPSED_KEY,
    currentAccountIdKey: WORKBUDDY_AI_CURRENT_ACCOUNT_ID_KEY,
    exportFilePrefix: 'workbuddy_ai_accounts',
    oauthTabKeys: ['oauth'],
    store: {
      accounts: store.accounts,
      currentAccountId: store.currentAccountId,
      loading: store.loading,
      error: store.error,
      fetchAccounts: store.fetchAccounts,
      fetchCurrentAccountId: store.fetchCurrentAccountId,
      deleteAccounts: store.deleteAccounts,
      refreshToken: store.refreshToken,
      refreshAllTokens: store.refreshAllTokens,
      setCurrentAccountId: store.setCurrentAccountId,
      updateAccountTags: store.updateAccountTags,
    },
    oauthService: {
      startLogin: workbuddyAiService.startWorkbuddyAiOAuthLogin,
      completeLogin: workbuddyAiService.completeWorkbuddyAiOAuthLogin,
      cancelLogin: workbuddyAiService.cancelWorkbuddyAiOAuthLogin,
    },
    dataService: {
      importFromJson: workbuddyAiService.importWorkbuddyAiFromJson,
      importFromLocal: workbuddyAiService.importWorkbuddyAiFromLocal,
      addWithToken: workbuddyAiService.addWorkbuddyAiAccountWithToken,
      exportAccounts: workbuddyAiService.exportWorkbuddyAiAccounts,
      injectToVSCode: workbuddyAiService.injectWorkbuddyAiToVSCode,
    },
    getDisplayEmail: (account) => getWorkbuddyAccountDisplayEmail(account),
  });

  return (
    <div className={`ghcp-accounts-page ${workbuddyAiPlatformConfig.pageClassName}`}>
      <CodebuddySuiteAccountsSharedView
        accounts={store.accounts}
        loading={store.loading}
        page={page}
        platformConfig={workbuddyAiPlatformConfig}
        onRefreshAccounts={() => {
          store.fetchAccounts();
        }}
      />
    </div>
  );
}
