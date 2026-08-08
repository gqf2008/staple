//! Root layout: HTML document, localized navigation, token layer, and the
//! language switcher.

use topcoat::{
    Result,
    context::{Cx, app_context},
    router::layout,
    view::view,
};

use crate::{
    attention::{AttentionQuery, build_attention_feed},
    i18n::{Lang, lang_code, lang_from_request, t, with_lang},
    routes::sidebar_badges::sidebar_badges_for,
    state::AppState,
};

use super::icons::{
    ACTIVITY, ALERT_CIRCLE, AWARD, BRIEFCASE, CHECK_CIRCLE, CLOCK, CODE, DATABASE, FILE_TEXT,
    FOLDER, GIT_BRANCH, GRID, INBOX, LAYOUT, LIST, SETTINGS, SHIELD, USER, USERS, ZAP,
};
use super::styles::TOKENS_CSS;
use topcoat::icon::icon;

/// Wraps every page in a full document with the design token layer, the
/// localized nav, and the language switcher.
#[layout("/")]
pub async fn root(cx: &Cx, slot: Result) -> Result {
    let lang = lang_from_request(cx);
    let parts = topcoat::context::try_request_context::<http::request::Parts>(cx);
    // Flash toast: rendered from the `?flash=` code left by mutating form
    // handlers (issue #231); auto-dismissed by ui_feedback.js.
    let flash = parts.as_ref().and_then(|parts| {
        parts.uri.query().and_then(|query| {
            query.split('&').find_map(|pair| {
                let (key, value) = pair.split_once('=')?;
                (key == "flash").then_some(value.to_owned())
            })
        })
    });
    let (flash_kind, flash_key) = match flash.as_deref() {
        Some("created") => ("success", "flash.created"),
        Some("saved") => ("success", "flash.saved"),
        Some("updated") => ("success", "flash.updated"),
        Some("triggered") => ("success", "flash.triggered"),
        Some("comment-added") => ("success", "flash.comment_added"),
        Some("decided") => ("success", "flash.decided"),
        Some("claimed") => ("success", "flash.claimed"),
        Some("archived") => ("success", "flash.archived"),
        Some("unarchived") => ("success", "flash.unarchived"),
        Some("dismissed") => ("success", "flash.dismissed"),
        Some("invalid") => ("error", "flash.invalid"),
        Some("error") | Some(_) => ("error", "flash.error"),
        None => ("", ""),
    };
    let current_path = parts
        .map(|parts| parts.uri.path().to_owned())
        .unwrap_or_else(|| "/".to_owned());
    // Active nav state: exact match or section prefix (e.g. board/chat
    // highlights Board, /companies/{id}/issues/{issue} highlights Issues).
    // Returns None so the attribute is omitted entirely when not active.
    let active_for = |href: &str| -> Option<&'static str> {
        if current_path == href || current_path.starts_with(&format!("{href}/")) {
            Some("active")
        } else {
            None
        }
    };
    let brand_class = if current_path == "/" {
        "brand active".to_string()
    } else {
        "brand".to_string()
    };
    // The company overview page (/companies/{id}) is the landing page of the
    // company scope and highlights the Dashboard entry.
    let dashboard_active = |company_id: &str| -> Option<&'static str> {
        let dash = format!("/companies/{company_id}/dashboard");
        if current_path == format!("/companies/{company_id}")
            || current_path == dash
            || current_path.starts_with(&format!("{dash}/"))
        {
            Some("active")
        } else {
            None
        }
    };
    let switch_lang = match lang {
        Lang::En => Lang::ZhCn,
        Lang::ZhCn => Lang::ZhTw,
        Lang::ZhTw => Lang::En,
        _ => Lang::En,
    };
    let switch_label = match switch_lang {
        Lang::En => "English",
        Lang::ZhCn => "中文",
        Lang::ZhTw => "繁體",
        _ => "English",
    };
    let html_lang = lang_code(lang);
    let company_id = current_path
        .strip_prefix("/companies/")
        .and_then(|rest| rest.split('/').next())
        .map(str::to_owned);
    // Sidebar badges (best-effort; failures render without badges).
    let mut inbox_badge = 0usize;
    let mut approvals_badge = 0usize;
    let mut attention_badge = 0usize;
    if let Some(company_id) = &company_id {
        let state = app_context::<AppState>(cx);
        if let Ok(badges) = sidebar_badges_for(state, company_id).await {
            inbox_badge = badges.inbox;
            approvals_badge = badges.approvals;
        }
        if let Ok(feed) = build_attention_feed(
            state,
            company_id,
            &AttentionQuery {
                limit: 1,
                sort: "activity".to_owned(),
                ..AttentionQuery::default()
            },
        )
        .await
        {
            attention_badge = feed.desk_badge_count;
        }
    }
    view! {
        <!DOCTYPE html>
        <html lang=(html_lang)>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>(t(lang, "nav.title"))</title>
                <style>(TOKENS_CSS)</style>
                <script src="/static/theme_init.js"></script>
            </head>
            <body>
                <div class="app-shell">
                    <div id="sidebar-scrim" class="sidebar-scrim" aria-hidden="true" hidden="hidden"></div>
                    <button type="button" class="sidebar-toggle secondary" id="sidebar-toggle" aria-controls="app-sidebar" aria-label=(t(lang, "nav.collapse")) aria-expanded="true" data-collapse=(t(lang, "nav.collapse")) data-expand=(t(lang, "nav.expand"))>("\u{ab}")</button>
                    <nav class="app-sidebar" id="app-sidebar" data-collapsible="true">
                        <button type="button" class="sidebar-toggle secondary" id="theme-toggle" aria-label=(t(lang, "nav.themeSystem")) data-theme-system=(t(lang, "nav.themeSystem")) data-theme-light=(t(lang, "nav.themeLight")) data-theme-dark=(t(lang, "nav.themeDark"))>("\u{25d0}")</button>
                        <div class="sidebar-resizer" id="sidebar-resizer" role="separator" aria-orientation="vertical" aria-label=(t(lang, "nav.resize")) tabindex="0" aria-valuemin="208" aria-valuemax="420" aria-valuenow="240"></div>
                        <a class=(brand_class) href=(with_lang("/", lang)) aria-label=(t(lang, "nav.title"))>icon(data: SHIELD) <span class="nav-label">(t(lang, "nav.title"))</span></a>
                        <h3>(t(lang, "nav.companies"))</h3>
                        <a href=(with_lang("/", lang)) class=(active_for("/")) aria-label=(t(lang, "nav.companies"))>icon(data: GRID) <span class="nav-label">(t(lang, "nav.companies"))</span></a>
                        if let Some(company_id) = &company_id {
                            <h3>(t(lang, "nav.board"))</h3>
                            <a href=(with_lang(&format!("/companies/{company_id}/dashboard"), lang)) class=(dashboard_active(company_id)) aria-label=(t(lang, "dashboard.title"))>icon(data: LAYOUT) <span class="nav-label">(t(lang, "dashboard.title"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/board"), lang)) class=(active_for(&format!("/companies/{company_id}/board"))) aria-label=(t(lang, "nav.board"))>icon(data: GRID) <span class="nav-label">(t(lang, "nav.board"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/inbox"), lang)) class=(active_for(&format!("/companies/{company_id}/inbox"))) aria-label=(t(lang, "inbox.title"))>
                                icon(data: INBOX)
                                <span class="nav-label">
                                    (t(lang, "inbox.title"))
                                    if inbox_badge > 0 {
                                        " " <span class="badge badge-default">(inbox_badge.to_string())</span>
                                    }
                                </span>
                            </a>
                            <a href=(with_lang(&format!("/companies/{company_id}/what-needs-me"), lang)) class=(active_for(&format!("/companies/{company_id}/what-needs-me"))) aria-label=(t(lang, "whatNeedsMe.title"))>
                                icon(data: ALERT_CIRCLE)
                                <span class="nav-label">
                                    (t(lang, "whatNeedsMe.title"))
                                    if attention_badge > 0 {
                                        " " <span class="badge badge-default">(attention_badge.to_string())</span>
                                    }
                                </span>
                            </a>
                            <a href=(with_lang(&format!("/companies/{company_id}/approvals"), lang)) class=(active_for(&format!("/companies/{company_id}/approvals"))) aria-label=(t(lang, "approvals.title"))>
                                icon(data: CHECK_CIRCLE)
                                <span class="nav-label">
                                    (t(lang, "approvals.title"))
                                    if approvals_badge > 0 {
                                        " " <span class="badge badge-default">(approvals_badge.to_string())</span>
                                    }
                                </span>
                            </a>
                            <h3>(t(lang, "nav.work"))</h3>
                            <a href=(with_lang(&format!("/companies/{company_id}/issues"), lang)) class=(active_for(&format!("/companies/{company_id}/issues"))) aria-label=(t(lang, "nav.issues"))>icon(data: LIST) <span class="nav-label">(t(lang, "nav.issues"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/pipelines"), lang)) class=(active_for(&format!("/companies/{company_id}/pipelines"))) aria-label=(t(lang, "pipelines.title"))>icon(data: GIT_BRANCH) <span class="nav-label">(t(lang, "pipelines.title"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/routines"), lang)) class=(active_for(&format!("/companies/{company_id}/routines"))) aria-label=(t(lang, "routines.title"))>icon(data: CLOCK) <span class="nav-label">(t(lang, "routines.title"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/artifacts"), lang)) class=(active_for(&format!("/companies/{company_id}/artifacts"))) aria-label=(t(lang, "artifacts.title"))>icon(data: FILE_TEXT) <span class="nav-label">(t(lang, "artifacts.title"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/skills"), lang)) class=(active_for(&format!("/companies/{company_id}/skills"))) aria-label=(t(lang, "skills.title"))>icon(data: CODE) <span class="nav-label">(t(lang, "skills.title"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/projects"), lang)) class=(active_for(&format!("/companies/{company_id}/projects"))) aria-label=(t(lang, "projects.title"))>icon(data: FOLDER) <span class="nav-label">(t(lang, "projects.title"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/agents"), lang)) class=(active_for(&format!("/companies/{company_id}/agents"))) aria-label=(t(lang, "agents.title"))>icon(data: USERS) <span class="nav-label">(t(lang, "agents.title"))</span></a>
                            <h3>(t(lang, "nav.company"))</h3>
                            <a href=(with_lang(&format!("/companies/{company_id}/org-chart"), lang)) class=(active_for(&format!("/companies/{company_id}/org-chart"))) aria-label=(t(lang, "orgChart.title"))>icon(data: SHIELD) <span class="nav-label">(t(lang, "orgChart.title"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/timeline"), lang)) class=(active_for(&format!("/companies/{company_id}/timeline"))) aria-label=(t(lang, "timeline.title"))>icon(data: ACTIVITY) <span class="nav-label">(t(lang, "timeline.title"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/costs"), lang)) class=(active_for(&format!("/companies/{company_id}/costs"))) aria-label=(t(lang, "costs.title"))>icon(data: BRIEFCASE) <span class="nav-label">(t(lang, "costs.title"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/activity"), lang)) class=(active_for(&format!("/companies/{company_id}/activity"))) aria-label=(t(lang, "activity.title"))>icon(data: ZAP) <span class="nav-label">(t(lang, "activity.title"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/access"), lang)) class=(active_for(&format!("/companies/{company_id}/access"))) aria-label=(t(lang, "access.title"))>icon(data: USER) <span class="nav-label">(t(lang, "access.title"))</span></a>
                            <a href=(with_lang(&format!("/companies/{company_id}/settings"), lang)) class=(active_for(&format!("/companies/{company_id}/settings"))) aria-label=(t(lang, "settings.title"))>icon(data: SETTINGS) <span class="nav-label">(t(lang, "settings.title"))</span></a>
                        }
                        <h3>(t(lang, "instance.title"))</h3>
                        <a href=(with_lang("/instance/settings", lang)) class=(active_for("/instance/settings")) aria-label=(t(lang, "instance.title"))>icon(data: DATABASE) <span class="nav-label">(t(lang, "instance.title"))</span></a>
                        <a href=(with_lang("/profile/settings", lang)) class=(active_for("/profile/settings")) aria-label=(t(lang, "profile.title"))>icon(data: USER) <span class="nav-label">(t(lang, "profile.title"))</span></a>
                        <a href=(with_lang("/adapters", lang)) class=(active_for("/adapters")) aria-label=(t(lang, "adapters.title"))>icon(data: ZAP) <span class="nav-label">(t(lang, "adapters.title"))</span></a>
                        <a href=(with_lang(&current_path, switch_lang)) aria-label=(switch_label)>icon(data: AWARD) <span class="nav-label">(switch_label)</span></a></nav>
                    <main class="app-main">(slot?)</main>
                </div>
                <div id="command-palette" class="command-palette" hidden="hidden" role="dialog" aria-modal="true" aria-label=(t(lang, "palette.placeholder"))>
                    <div class="command-palette-panel" data-company-id=(company_id.clone().unwrap_or_default())>
                        <input id="command-input" class="command-palette-input" type="text"
                               placeholder=(t(lang, "palette.placeholder")) autocomplete="off">
                        <div id="command-list" class="command-palette-list">
                            <a class="command-item" href=(with_lang("/", lang))>(t(lang, "nav.companies"))</a>
                            if let Some(company_id) = &company_id {
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/board"), lang))>(t(lang, "nav.board"))</a>
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/issues"), lang))>(t(lang, "nav.issues"))</a>
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/agents"), lang))>(t(lang, "agents.title"))</a>
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/goals"), lang))>(t(lang, "nav.goals"))</a>
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/projects"), lang))>(t(lang, "nav.projects"))</a>
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/routines"), lang))>(t(lang, "routines.title"))</a>
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/skills"), lang))>(t(lang, "settings.skills"))</a>
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/approvals"), lang))>(t(lang, "approvals.title"))</a>
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/inbox"), lang))>(t(lang, "inbox.title"))</a>
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/what-needs-me"), lang))>(t(lang, "whatNeedsMe.title"))</a>
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/decisions"), lang))>(t(lang, "decisions.title"))</a>
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/costs"), lang))>(t(lang, "costs.title"))</a>
                                <a class="command-item" href=(with_lang(&format!("/companies/{company_id}/settings"), lang))>(t(lang, "settings.title"))</a>
                            }
                            <a class="command-item" href=(with_lang("/instance/settings", lang))>(t(lang, "instance.title"))</a>
                            <a class="command-item" href=(with_lang("/profile/settings", lang))>(t(lang, "profile.title"))</a>
                            <a class="command-item" href=(with_lang("/adapters", lang))>(t(lang, "adapters.title"))</a>
                            <div id="command-empty" class="command-empty" hidden="hidden" role="status">(t(lang, "palette.empty"))</div>
                        </div>
                    </div>
                </div>
                if !flash_kind.is_empty() {
                    <div id="flash-toast" class=(format!("toast toast-{flash_kind}")) role="status" aria-live="polite">
                        (t(lang, flash_key))
                    </div>
                }
                <script src="/static/board_chat.js"></script>
                <script src="/static/board_zip.js"></script>
                <script src="/static/command_palette.js"></script>
                <script src="/static/ui_feedback.js"></script>
                <script src="/static/sidebar.js"></script>
                <script src="/static/theme.js"></script>
            </body>
        </html>
    }
}
