//! Lightweight UI internationalization.
//!
//! Locale resources are flat key → string maps (en + zh-CN). Pages pick the
//! language from the `?lang=` query parameter; every user-visible string in
//! the Topcoat UI goes through [`t`]. The key set mirrors the surface that
//! upstream `ui/src/i18n` covers for the board pages.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Supported languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    /// English (default).
    En,
    /// Simplified Chinese.
    ZhCn,
    /// Traditional Chinese.
    ZhTw,
}

/// Current default language.
pub const DEFAULT_LANG: Lang = Lang::En;

/// Parses a language from a `?lang=` value (`zh-CN`, `zh` → [`Lang::ZhCn`];
/// `zh-TW` → [`Lang::ZhTw`]; anything else → [`Lang::En`]).
#[must_use]
pub fn parse_lang(value: Option<&str>) -> Lang {
    match value {
        Some("zh-CN") | Some("zh") => Lang::ZhCn,
        Some("zh-TW") => Lang::ZhTw,
        _ => Lang::En,
    }
}

/// Reads the language from the request URI query string.
#[must_use]
pub fn lang_from_request(cx: &topcoat::context::Cx) -> Lang {
    let parts = topcoat::context::try_request_context::<http::request::Parts>(cx);
    let lang = parts.and_then(|parts| {
        parts.uri.query().and_then(|query| {
            query.split('&').find_map(|pair| {
                let (key, value) = pair.split_once('=')?;
                (key == "lang").then_some(value)
            })
        })
    });
    parse_lang(lang)
}

/// Translates `key` into `lang`. Resolution order:
/// 1. the imported upstream locale table (full keyset, en/zh-CN/zh-TW);
/// 2. the legacy local table (kept for Staple-specific keys);
/// 3. the key itself.
#[must_use]
pub fn t(lang: Lang, key: &str) -> String {
    let table = match lang {
        Lang::En => &UPSTREAM_EN,
        Lang::ZhCn => &UPSTREAM_ZH_CN,
        Lang::ZhTw => &UPSTREAM_ZH_TW,
    };
    if let Some(value) = table.get(key) {
        return value.clone();
    }
    let legacy = match lang {
        Lang::En => &EN,
        Lang::ZhCn => &ZH_CN,
        // zh-TW falls back to the zh-CN local table for Staple-specific keys.
        Lang::ZhTw => &ZH_CN,
    };
    legacy.get(key).copied().unwrap_or(key).to_owned()
}

/// Appends or replaces the `lang` query parameter on a path.
#[must_use]
pub fn with_lang(path: &str, lang: Lang) -> String {
    let value = match lang {
        Lang::En => "en",
        Lang::ZhCn => "zh-CN",
        Lang::ZhTw => "zh-TW",
    };
    if let Some((base, query)) = path.split_once('?') {
        let rest: Vec<&str> = query
            .split('&')
            .filter(|pair| !pair.starts_with("lang="))
            .collect();
        if rest.is_empty() {
            format!("{base}?lang={value}")
        } else {
            format!("{base}?{}&lang={value}", rest.join("&"))
        }
    } else {
        format!("{path}?lang={value}")
    }
}

/// Returns the current language code (`en` / `zh-CN`).
#[must_use]
pub fn lang_code(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "en",
        Lang::ZhCn => "zh-CN",
        Lang::ZhTw => "zh-TW",
    }
}

/// Flattens a nested locale JSON object into `key` → value.
fn flatten_locale(value: &serde_json::Value) -> HashMap<String, String> {
    fn walk(value: &serde_json::Value, prefix: &str, out: &mut HashMap<String, String>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, child) in map {
                    let next = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}.{key}")
                    };
                    walk(child, &next, out);
                }
            }
            serde_json::Value::String(text) => {
                out.insert(prefix.to_owned(), text.clone());
            }
            _ => {}
        }
    }
    let mut out = HashMap::new();
    walk(value, "", &mut out);
    out
}

/// Loads an upstream locale file embedded at compile time.
fn load_locale(source: &str) -> HashMap<String, String> {
    serde_json::from_str::<serde_json::Value>(source)
        .map(|value| flatten_locale(&value))
        .unwrap_or_default()
}

/// Upstream `en` locale (full keyset, ~10k keys).
static UPSTREAM_EN: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| load_locale(include_str!("../locales/en.json")));
/// Upstream `zh-CN` locale.
static UPSTREAM_ZH_CN: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| load_locale(include_str!("../locales/zh-CN.json")));
/// Upstream `zh-TW` locale.
static UPSTREAM_ZH_TW: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| load_locale(include_str!("../locales/zh-TW.json")));

static EN: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    entries(&[
        ("nav.title", "Staple"),
        ("nav.companies", "Companies"),
        ("page.title.companies", "Companies"),
        (
            "empty.noCompanies",
            "No companies yet. Create one via the API.",
        ),
        ("section.goals", "Goals"),
        ("section.projects", "Projects"),
        ("section.issues", "Issues"),
        ("empty.noGoals", "No goals."),
        ("empty.noProjects", "No projects."),
        ("empty.noIssues", "No issues."),
        ("page.title.issues", "Issues"),
        ("meta.company", "company"),
        ("meta.assignee", "assignee"),
        ("meta.priority", "priority"),
        ("issue.comments", "Comments"),
        ("empty.noComments", "No comments."),
        ("issue.documents", "Documents"),
        ("empty.noDocuments", "No documents."),
        ("empty.noAttachments", "No attachments."),
        ("empty.noWorkProducts", "No work products."),
        ("issue.attachments", "Attachments"),
        ("issue.workProducts", "Work products"),
        ("issue.addComment", "Add a comment"),
        ("issue.commentPlaceholder", "Add a comment"),
        ("issue.add", "Add"),
        ("issue.rev", "rev"),
        ("issue.untitled", "untitled"),
        ("approvals.title", "Approvals"),
        ("approvals.request", "Request"),
        ("approvals.pending", "Pending"),
        ("approvals.approve", "Approve"),
        ("approvals.reject", "Reject"),
        ("approvals.noApprovals", "No approvals."),
        ("activity.title", "Audit log"),
        ("activity.noActivity", "No activity."),
        ("nav.board", "Board"),
        ("nav.issues", "Issues"),
        ("nav.search", "Search"),
        ("nav.approvals", "Approvals"),
        ("nav.activity", "Activity"),
        ("nav.settings", "Settings"),
        ("board.title", "Board"),
        ("board.move", "Move"),
        ("search.title", "Search"),
        ("search.placeholder", "Search issues by title or identifier"),
        ("search.submit", "Search"),
        ("search.noResults", "No matching issues."),
        ("settings.title", "Settings"),
        ("settings.company", "Company"),
        ("settings.budget", "Budget"),
        ("settings.secrets", "Secrets"),
        ("settings.skills", "Skills"),
        ("settings.save", "Save"),
        ("settings.add", "Add"),
        ("settings.noSecrets", "No secrets."),
        ("settings.noSkills", "No skills."),
        ("settings.secretName", "Name"),
        ("settings.secretValue", "Value"),
        ("settings.skillName", "Name"),
        ("settings.skillDescription", "Description"),
        ("agents.title", "Agents"),
        ("agents.noAgents", "No agents."),
        ("agent.pauseReason", "Pause reason"),
        ("agent.pause", "Pause"),
        ("agent.resume", "Resume"),
        ("agent.runtime", "Runtime state"),
        ("agent.session", "Session"),
        ("agent.lastRunStatus", "Last run"),
        ("agent.tokens", "Tokens"),
        ("agent.cost", "Cost"),
        ("agent.noRuntime", "No runtime state."),
        ("agent.sessions", "Task sessions"),
        ("agent.noSessions", "No sessions."),
        ("agent.wakeups", "Wakeups"),
        ("agent.noWakeups", "No wakeups."),
        ("agent.budget", "Budget"),
        ("agent.monthlyBudget", "Monthly budget"),
        ("agent.spent", "Spent"),
        ("decision.title", "Decision desk"),
        ("decision.queues", "Queues"),
        ("decision.queueName", "Queue name"),
        ("decision.noQueues", "No queues."),
        ("decision.triage", "Triage"),
        ("decision.noTriage", "No triage."),
        ("decision.retention", "Retention"),
        ("decision.noRetention", "No retention rows."),
        ("decision.restore", "Restore"),
        ("decision.outbox", "Archive notifications"),
        ("decision.noOutbox", "No notifications."),
        ("inbox.title", "Inbox"),
        ("inbox.empty", "Inbox is empty."),
        ("inbox.archive", "Archive"),
        ("access.title", "Access"),
        ("access.members", "Members"),
        ("access.noMembers", "No members."),
        ("access.invites", "Invites"),
        ("access.inviteName", "Name"),
        ("access.invite", "Invite"),
        ("access.noInvites", "No invites."),
        ("access.revoke", "Revoke"),
        ("access.joinRequests", "Join requests"),
        ("access.noJoinRequests", "No join requests."),
        ("access.approve", "Approve"),
        ("access.reject", "Reject"),
        ("access.grants", "Permission grants"),
        ("access.noGrants", "No grants."),
        ("costs.title", "Costs"),
        ("costs.summary", "Summary"),
        ("costs.budget", "Budget"),
        ("costs.spent", "Spent"),
        ("costs.pausedAgents", "Paused agents"),
        ("costs.byAgent", "By agent"),
        ("costs.noRows", "No cost rows."),
        ("routines.title", "Routines"),
        ("routines.noRoutines", "No routines."),
        ("routines.rev", "rev"),
        ("routines.trigger", "Trigger"),
        ("instance.title", "Instance settings"),
        ("instance.roles", "Instance roles"),
        ("instance.userId", "User id"),
        ("instance.noRoles", "No roles."),
        ("instance.boardKeys", "Board API keys"),
        ("instance.keyName", "Key name"),
        ("instance.noKeys", "No keys."),
        ("instance.challenges", "CLI auth challenges"),
        ("instance.challengeCommand", "Command"),
        ("instance.challenge", "Challenge"),
        ("instance.noChallenges", "No challenges."),
        ("dashboard.title", "Dashboard"),
        ("dashboard.issues", "Issues"),
        ("dashboard.total", "Total"),
        ("dashboard.agents", "Agents"),
        ("dashboard.budget", "Budget"),
        ("dashboard.activity", "Recent activity"),
        ("projects.edit", "Edit project"),
        ("projects.issues", "Issues"),
        ("workspaces.title", "Workspaces"),
        ("workspaces.project", "Project workspaces"),
        ("workspaces.execution", "Execution workspaces"),
        ("workspaces.services", "Runtime services"),
        ("workspaces.operations", "Workspace operations"),
        ("workspaces.empty", "Nothing here."),
        ("workspaces.materialize", "Materialize"),
        ("adapters.title", "Adapters"),
        ("adapters.registered", "Registered adapters"),
        ("adapters.plugins", "Plugin diagnostics"),
        ("adapters.empty", "No adapters."),
        ("adapters.noPlugins", "No plugin reports."),
        ("common.language", "Language"),
        ("common.back", "Back"),
    ])
});

static ZH_CN: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    entries(&[
        ("nav.title", "Staple"),
        ("nav.companies", "公司"),
        ("page.title.companies", "公司"),
        ("empty.noCompanies", "暂无公司。可通过 API 创建。"),
        ("section.goals", "目标"),
        ("section.projects", "项目"),
        ("section.issues", "任务"),
        ("empty.noGoals", "暂无目标。"),
        ("empty.noProjects", "暂无项目。"),
        ("empty.noIssues", "暂无任务。"),
        ("page.title.issues", "任务"),
        ("meta.company", "公司"),
        ("meta.assignee", "负责人"),
        ("meta.priority", "优先级"),
        ("issue.comments", "评论"),
        ("empty.noComments", "暂无评论。"),
        ("issue.documents", "文档"),
        ("empty.noDocuments", "暂无文档。"),
        ("empty.noAttachments", "暂无附件。"),
        ("empty.noWorkProducts", "暂无工作产物。"),
        ("issue.attachments", "附件"),
        ("issue.workProducts", "工作产物"),
        ("issue.addComment", "发表评论"),
        ("issue.commentPlaceholder", "发表评论"),
        ("issue.add", "添加"),
        ("issue.rev", "版本"),
        ("issue.untitled", "未命名"),
        ("approvals.title", "审批"),
        ("approvals.request", "发起"),
        ("approvals.pending", "待处理"),
        ("approvals.approve", "通过"),
        ("approvals.reject", "拒绝"),
        ("approvals.noApprovals", "暂无审批。"),
        ("activity.title", "审计日志"),
        ("activity.noActivity", "暂无活动。"),
        ("nav.board", "看板"),
        ("nav.issues", "任务"),
        ("nav.search", "搜索"),
        ("nav.approvals", "审批"),
        ("nav.activity", "审计"),
        ("nav.settings", "设置"),
        ("board.title", "看板"),
        ("board.move", "移动"),
        ("search.title", "搜索"),
        ("search.placeholder", "按标题或编号搜索任务"),
        ("search.submit", "搜索"),
        ("search.noResults", "没有匹配的任务。"),
        ("settings.title", "设置"),
        ("settings.company", "公司"),
        ("settings.budget", "预算"),
        ("settings.secrets", "密钥"),
        ("settings.skills", "技能"),
        ("settings.save", "保存"),
        ("settings.add", "添加"),
        ("settings.noSecrets", "暂无密钥。"),
        ("settings.noSkills", "暂无技能。"),
        ("settings.secretName", "名称"),
        ("settings.secretValue", "值"),
        ("settings.skillName", "名称"),
        ("settings.skillDescription", "描述"),
        ("agents.title", "智能体"),
        ("agents.noAgents", "暂无智能体。"),
        ("agent.pauseReason", "暂停原因"),
        ("agent.pause", "暂停"),
        ("agent.resume", "恢复"),
        ("agent.runtime", "运行时状态"),
        ("agent.session", "会话"),
        ("agent.lastRunStatus", "上次运行"),
        ("agent.tokens", "Token"),
        ("agent.cost", "成本"),
        ("agent.noRuntime", "暂无运行时状态。"),
        ("agent.sessions", "任务会话"),
        ("agent.noSessions", "暂无会话。"),
        ("agent.wakeups", "唤醒"),
        ("agent.noWakeups", "暂无唤醒。"),
        ("agent.budget", "预算"),
        ("agent.monthlyBudget", "月度预算"),
        ("agent.spent", "已花费"),
        ("decision.title", "决策桌"),
        ("decision.queues", "队列"),
        ("decision.queueName", "队列名称"),
        ("decision.noQueues", "暂无队列。"),
        ("decision.triage", "分流"),
        ("decision.noTriage", "暂无分流记录。"),
        ("decision.retention", "保留"),
        ("decision.noRetention", "暂无保留记录。"),
        ("decision.restore", "恢复"),
        ("decision.outbox", "归档通知"),
        ("decision.noOutbox", "暂无通知。"),
        ("inbox.title", "收件箱"),
        ("inbox.empty", "收件箱为空。"),
        ("inbox.archive", "归档"),
        ("access.title", "访问"),
        ("access.members", "成员"),
        ("access.noMembers", "暂无成员。"),
        ("access.invites", "邀请"),
        ("access.inviteName", "名称"),
        ("access.invite", "邀请"),
        ("access.noInvites", "暂无邀请。"),
        ("access.revoke", "撤销"),
        ("access.joinRequests", "加入申请"),
        ("access.noJoinRequests", "暂无加入申请。"),
        ("access.approve", "批准"),
        ("access.reject", "拒绝"),
        ("access.grants", "权限授权"),
        ("access.noGrants", "暂无授权。"),
        ("costs.title", "成本"),
        ("costs.summary", "汇总"),
        ("costs.budget", "预算"),
        ("costs.spent", "已花费"),
        ("costs.pausedAgents", "已暂停智能体"),
        ("costs.byAgent", "按智能体"),
        ("costs.noRows", "暂无成本记录。"),
        ("routines.title", "例行任务"),
        ("routines.noRoutines", "暂无例行任务。"),
        ("routines.rev", "修订"),
        ("routines.trigger", "触发"),
        ("instance.title", "实例设置"),
        ("instance.roles", "实例角色"),
        ("instance.userId", "用户 ID"),
        ("instance.noRoles", "暂无角色。"),
        ("instance.boardKeys", "Board API 密钥"),
        ("instance.keyName", "密钥名称"),
        ("instance.noKeys", "暂无密钥。"),
        ("instance.challenges", "CLI 认证挑战"),
        ("instance.challengeCommand", "命令"),
        ("instance.challenge", "发起挑战"),
        ("instance.noChallenges", "暂无挑战。"),
        ("dashboard.title", "仪表盘"),
        ("dashboard.issues", "任务"),
        ("dashboard.total", "合计"),
        ("dashboard.agents", "智能体"),
        ("dashboard.budget", "预算"),
        ("dashboard.activity", "最近活动"),
        ("projects.edit", "编辑项目"),
        ("projects.issues", "任务"),
        ("workspaces.title", "工作区"),
        ("workspaces.project", "项目工作区"),
        ("workspaces.execution", "执行工作区"),
        ("workspaces.services", "运行时服务"),
        ("workspaces.operations", "工作区操作"),
        ("workspaces.empty", "暂无内容。"),
        ("workspaces.materialize", "物化"),
        ("adapters.title", "适配器"),
        ("adapters.registered", "已注册适配器"),
        ("adapters.plugins", "插件诊断"),
        ("adapters.empty", "暂无适配器。"),
        ("adapters.noPlugins", "暂无插件报告。"),
        ("common.language", "语言"),
        ("common.back", "返回"),
    ])
});

fn entries(pairs: &[(&'static str, &'static str)]) -> HashMap<&'static str, &'static str> {
    pairs.iter().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_language_codes() {
        assert_eq!(parse_lang(None), Lang::En);
        assert_eq!(parse_lang(Some("en")), Lang::En);
        assert_eq!(parse_lang(Some("zh-CN")), Lang::ZhCn);
        assert_eq!(parse_lang(Some("zh")), Lang::ZhCn);
        assert_eq!(parse_lang(Some("fr")), Lang::En);
    }

    #[test]
    fn translates_keys() {
        assert_eq!(t(Lang::En, "section.issues"), "Issues");
        assert_eq!(t(Lang::ZhCn, "section.issues"), "任务");
        // Unknown keys fall back to the key itself.
        assert_eq!(t(Lang::ZhCn, "missing.key"), "missing.key");
    }

    #[test]
    fn appends_and_preserves_lang_query() {
        assert_eq!(with_lang("/", Lang::ZhCn), "/?lang=zh-CN");
        assert_eq!(
            with_lang("/companies/c1", Lang::En),
            "/companies/c1?lang=en"
        );
        assert_eq!(with_lang("/?lang=zh-CN", Lang::En), "/?lang=en");
        assert_eq!(
            with_lang("/issues/i1?x=1&lang=zh-CN", Lang::En),
            "/issues/i1?x=1&lang=en"
        );
    }
}
