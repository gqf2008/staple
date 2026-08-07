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

use super::styles::TOKENS_CSS;

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
                    <nav class="app-sidebar" data-collapsible="true">
                        <button type="button" class="sidebar-toggle secondary" id="sidebar-toggle" aria-controls="app-sidebar" aria-label=(t(lang, "nav.collapse")) aria-expanded="true" data-collapse=(t(lang, "nav.collapse")) data-expand=(t(lang, "nav.expand"))>("\u{ab}")</button>
                        <button type="button" class="sidebar-toggle secondary" id="theme-toggle" aria-label=(t(lang, "nav.themeSystem")) data-theme-system=(t(lang, "nav.themeSystem")) data-theme-light=(t(lang, "nav.themeLight")) data-theme-dark=(t(lang, "nav.themeDark"))>("\u{25d0}")</button>
                        <div class="sidebar-resizer" id="sidebar-resizer" role="separator" aria-orientation="vertical" aria-label=(t(lang, "nav.resize"))></div>
                        <a class="brand" href=(with_lang("/", lang))>(t(lang, "nav.title"))</a>
                        <h3>(t(lang, "nav.companies"))</h3>
                        <a href=(with_lang("/", lang))>(t(lang, "nav.companies"))</a>
                        if let Some(company_id) = &company_id {
                            <h3>(t(lang, "nav.board"))</h3>
                            <a href=(with_lang(&format!("/companies/{company_id}/dashboard"), lang))>(t(lang, "dashboard.title"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/board"), lang))>(t(lang, "nav.board"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/inbox"), lang))>
                                (t(lang, "inbox.title"))
                                if inbox_badge > 0 {
                                    " " <span class="badge badge-default">(inbox_badge.to_string())</span>
                                }
                            </a>
                            <a href=(with_lang(&format!("/companies/{company_id}/what-needs-me"), lang))>
                                (t(lang, "whatNeedsMe.title"))
                                if attention_badge > 0 {
                                    " " <span class="badge badge-default">(attention_badge.to_string())</span>
                                }
                            </a>
                            <a href=(with_lang(&format!("/companies/{company_id}/approvals"), lang))>
                                (t(lang, "approvals.title"))
                                if approvals_badge > 0 {
                                    " " <span class="badge badge-default">(approvals_badge.to_string())</span>
                                }
                            </a>
                            <h3>(t(lang, "nav.work"))</h3>
                            <a href=(with_lang(&format!("/companies/{company_id}/issues"), lang))>(t(lang, "nav.issues"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/pipelines"), lang))>(t(lang, "pipelines.title"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/routines"), lang))>(t(lang, "routines.title"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/artifacts"), lang))>(t(lang, "artifacts.title"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/skills"), lang))>(t(lang, "skills.title"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/projects"), lang))>(t(lang, "projects.title"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/agents"), lang))>(t(lang, "agents.title"))</a>
                            <h3>(t(lang, "nav.company"))</h3>
                            <a href=(with_lang(&format!("/companies/{company_id}/org-chart"), lang))>(t(lang, "orgChart.title"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/timeline"), lang))>(t(lang, "timeline.title"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/costs"), lang))>(t(lang, "costs.title"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/activity"), lang))>(t(lang, "activity.title"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/access"), lang))>(t(lang, "access.title"))</a>
                            <a href=(with_lang(&format!("/companies/{company_id}/settings"), lang))>(t(lang, "settings.title"))</a>
                        }
                        <h3>(t(lang, "instance.title"))</h3>
                        <a href=(with_lang("/instance/settings", lang))>(t(lang, "instance.title"))</a>
                        <a href=(with_lang("/profile/settings", lang))>(t(lang, "profile.title"))</a>
                        <a href=(with_lang("/adapters", lang))>(t(lang, "adapters.title"))</a>
                        <a href=(with_lang(&current_path, switch_lang))>(switch_label)</a>
                    </nav>
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
