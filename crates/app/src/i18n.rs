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
}

/// Current default language.
pub const DEFAULT_LANG: Lang = Lang::En;

/// Parses a language from a `?lang=` value (`zh-CN`, `zh` → [`Lang::ZhCn`];
/// anything else → [`Lang::En`]).
#[must_use]
pub fn parse_lang(value: Option<&str>) -> Lang {
    match value {
        Some("zh-CN") | Some("zh") => Lang::ZhCn,
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

/// Translates `key` into `lang`; unknown keys fall back to the key itself.
#[must_use]
pub fn t(lang: Lang, key: &str) -> String {
    let table = match lang {
        Lang::En => &EN,
        Lang::ZhCn => &ZH_CN,
    };
    table.get(key).copied().unwrap_or(key).to_owned()
}

/// Appends or replaces the `lang` query parameter on a path.
#[must_use]
pub fn with_lang(path: &str, lang: Lang) -> String {
    let value = match lang {
        Lang::En => "en",
        Lang::ZhCn => "zh-CN",
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
    }
}

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
