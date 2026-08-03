//! Board pages: company list, company overview, and issue list.

use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{page, path_param},
    view::view,
};

use crate::{
    i18n::{lang_from_request, t, with_lang},
    state::AppState,
};

fn to_topcoat_error(error: impl ToString) -> topcoat::Error {
    topcoat::Error::from(std::io::Error::other(error.to_string()))
}

/// Typed `{company_id}` path segment for UI pages.
#[path_param(error = bad_request("Invalid company id"))]
pub(crate) struct CompanyId(String);

/// Home: the company list (company selection context).
#[page("/")]
pub async fn home(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let state = app_context::<AppState>(cx);
    let companies = state.companies.list().await.map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "page.title.companies"))</h1>
        if companies.is_empty() {
            <p class="empty">(t(lang, "empty.noCompanies"))</p>
        } else {
            <ul class="list">
                for company in companies {
                    <li>
                        <a href=(with_lang(&format!("/companies/{}", company.id), lang))>
                            <strong>(company.name)</strong>
                        </a>
                        <span class="mono">" " (company.id)</span>
                    </li>
                }
            </ul>
        }
    }
}

/// Company overview: goals, projects, and issues for one company.
#[page("/companies/{company_id}")]
pub async fn company_overview(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company = state
        .companies
        .get(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let Some(company) = company else {
        return Err(topcoat::router::error::not_found().into());
    };
    let goals = state
        .goals
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let projects = state
        .projects
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let issues = state
        .issues
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;

    view! {
        <h1 class="page-title">(company.name)</h1>
        <p class="mono">(company.id)</p>

        <nav class="nav-row">
            <a href=(with_lang(&format!("/companies/{company_id}/board"), lang))>(t(lang, "nav.board"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/issues"), lang))>(t(lang, "nav.issues"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/search"), lang))>(t(lang, "nav.search"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/approvals"), lang))>(t(lang, "nav.approvals"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/activity"), lang))>(t(lang, "nav.activity"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/settings"), lang))>(t(lang, "nav.settings"))</a>
        </nav>

        <section>
            <h2>(t(lang, "section.goals"))</h2>
            if goals.is_empty() {
                <p class="empty">(t(lang, "empty.noGoals"))</p>
            } else {
                <ul class="list">
                    for goal in goals {
                        <li>
                            <strong>(goal.title)</strong>
                            <span class="badge badge-default">(goal.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>(t(lang, "section.projects"))</h2>
            if projects.is_empty() {
                <p class="empty">(t(lang, "empty.noProjects"))</p>
            } else {
                <ul class="list">
                    for project in projects {
                        <li>
                            <strong>(project.name)</strong>
                            <span class="badge badge-default">(project.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>(t(lang, "section.issues"))</h2>
            if issues.is_empty() {
                <p class="empty">(t(lang, "empty.noIssues"))</p>
            } else {
                <ul class="list">
                    for issue in issues {
                        <li>
                            <span class="mono">(issue.identifier.clone())</span>
                            " " <strong>(issue.title.clone())</strong>
                            " " <span class=(status_badge_class(&issue.status))>(issue.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Issue list page for one company.
#[page("/companies/{company_id}/issues")]
pub async fn company_issues(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let issues = state
        .issues
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "page.title.issues"))</h1>
        <ul class="list">
            for issue in issues {
                <li>
                    <span class="mono">(issue.identifier.clone())</span>
                    " " <strong>(issue.title.clone())</strong>
                    " " <span class=(status_badge_class(&issue.status))>(issue.status)</span>
                </li>
            }
        </ul>
    }
}

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "running" | "in_progress" => "badge badge-running",
        "paused" => "badge badge-paused",
        "blocked" => "badge badge-blocked",
        "done" | "completed" => "badge badge-done",
        _ => "badge badge-default",
    }
}

/// Issue detail: attributes, comments (with add form), documents,
/// attachments, and work products.
#[page("/issues/{id}")]
pub async fn issue_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(issue) = state
        .issues
        .get(&issue_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let comments = state
        .comments
        .list(&issue_id)
        .await
        .map_err(to_topcoat_error)?;
    let documents = state
        .documents
        .list_issue_documents(&issue_id)
        .await
        .map_err(to_topcoat_error)?;
    let attachments = state
        .assets
        .list_issue_attachments(&issue_id)
        .await
        .map_err(to_topcoat_error)?;
    let work_products = state
        .work_products
        .list_for_issue(&issue_id)
        .await
        .map_err(to_topcoat_error)?;

    view! {
        <h1 class="page-title">(issue.identifier) " " (issue.title)</h1>
        <p>
            <span class=(status_badge_class(&issue.status))>(issue.status)</span>
            " " (t(lang, "meta.priority")) ": " <span class="mono">(issue.priority)</span>
        </p>
        <p class="meta-row">
            (t(lang, "meta.company")) ": " (issue.company_id)
            " | " (t(lang, "meta.assignee")) ": "
            (issue.assignee_agent_id.as_deref().unwrap_or("-"))
        </p>
        if let Some(description) = &issue.description {
            <p>(description)</p>
        }

        <section>
            <h2>(t(lang, "issue.comments"))</h2>
            <form class="inline-form" method="post" action=(with_lang(&format!("/issues/{issue_id}/comments/ui"), lang))>
                <input type="text" name="body" placeholder=(t(lang, "issue.commentPlaceholder")) required="">
                <button type="submit">(t(lang, "issue.add"))</button>
            </form>
            if comments.is_empty() {
                <p class="empty">(t(lang, "empty.noComments"))</p>
            } else {
                <ul class="list">
                    for comment in comments {
                        <li>
                            <p class="meta-row">(comment.author_user_id.as_deref().unwrap_or("agent")) " @ " (comment.created_at)</p>
                            <p>(comment.body)</p>
                        </li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>(t(lang, "issue.documents"))</h2>
            if documents.is_empty() {
                <p class="empty">(t(lang, "empty.noDocuments"))</p>
            } else {
                <ul class="list">
                    for document in documents {
                        <li>
                            <strong>(document.title.as_deref().unwrap_or(&t(lang, "issue.untitled")))</strong>
                            " " (t(lang, "issue.rev")) " " <span class="mono">(document.latest_revision_number)</span>
                        </li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>(t(lang, "issue.attachments"))</h2>
            if attachments.is_empty() {
                <p class="empty">(t(lang, "empty.noAttachments"))</p>
            } else {
                <ul class="list">
                    for attachment in attachments {
                        <li><span class="mono">(attachment.asset_id)</span></li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>(t(lang, "issue.workProducts"))</h2>
            if work_products.is_empty() {
                <p class="empty">(t(lang, "empty.noWorkProducts"))</p>
            } else {
                <ul class="list">
                    for product in work_products {
                        <li>
                            <strong>(product.title)</strong>
                            " " <span class="mono">(product.r#type)</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Approvals page: create form + pending list with decide buttons.
#[page("/companies/{company_id}/approvals")]
pub async fn approvals(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let approvals = state
        .approvals
        .list(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;

    view! {
        <h1 class="page-title">(t(lang, "approvals.title"))</h1>

        <section>
            <h2>(t(lang, "approvals.request"))</h2>
            <form class="inline-form" method="post" action=(with_lang(&format!("/companies/{company_id}/approvals/ui"), lang))>
                <select name="type">
                    <option value="hire_agent">"hire_agent"</option>
                    <option value="approve_ceo_strategy">"approve_ceo_strategy"</option>
                    <option value="budget_override_required">"budget_override_required"</option>
                    <option value="request_board_approval">"request_board_approval"</option>
                </select>
                <input type="text" name="payload" placeholder="{}">
                <button type="submit">(t(lang, "approvals.request"))</button>
            </form>
        </section>

        <section>
            <h2>(t(lang, "approvals.pending"))</h2>
            if approvals.is_empty() {
                <p class="empty">(t(lang, "approvals.noApprovals"))</p>
            } else {
                <ul class="list">
                    for approval in approvals {
                        <li>
                            <strong>(&approval.r#type)</strong>
                            " " <span class=(status_badge_class(&approval.status))>(&approval.status)</span>
                            " " <span class="mono">(&approval.id)</span>
                            if approval.status == "pending" {
                                <form class="inline-form" method="post" action=(with_lang(&format!("/approvals/{}/decide/ui", approval.id), lang))>
                                    <input type="hidden" name="decision" value="approved">
                                    <button type="submit">(t(lang, "approvals.approve"))</button>
                                </form>
                                <form class="inline-form" method="post" action=(with_lang(&format!("/approvals/{}/decide/ui", approval.id), lang))>
                                    <input type="hidden" name="decision" value="rejected">
                                    <button type="submit" class="destructive">(t(lang, "approvals.reject"))</button>
                                </form>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Activity log view.
#[page("/companies/{company_id}/activity")]
pub async fn activity(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let entries = state
        .activity
        .list(&company_id, 200)
        .await
        .map_err(to_topcoat_error)?;

    view! {
        <h1 class="page-title">(t(lang, "activity.title"))</h1>
        if entries.is_empty() {
            <p class="empty">(t(lang, "activity.noActivity"))</p>
        } else {
            <ul class="list">
                for entry in entries {
                    <li>
                        <span class="mono">(entry.created_at)</span>
                        " " <strong>(entry.action)</strong>
                        " " <span class="meta-row">(entry.actor_type) "/" (entry.actor_id)</span>
                        " " <span class="mono">(entry.entity_type) ":" (entry.entity_id)</span>
                    </li>
                }
            </ul>
        }
    }
}

/// Board page: status columns with per-issue status moves.
#[page("/companies/{company_id}/board")]
pub async fn board(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let issues = state
        .issues
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let statuses = [
        "backlog",
        "todo",
        "in_progress",
        "in_review",
        "blocked",
        "done",
    ];
    let moveable = [
        "backlog",
        "todo",
        "in_progress",
        "in_review",
        "blocked",
        "done",
        "cancelled",
    ];
    view! {
        <h1 class="page-title">(t(lang, "board.title"))</h1>
        <a class="muted-link" href=(with_lang(&format!("/companies/{company_id}"), lang))>"← "</a>
        <div class="board-grid">
            for status in statuses {
                <section class="board-column">
                    <h2 class="board-column-title">
                        <span class=(status_badge_class(status))>(status)</span>
                        " " <span class="mono">(
                            issues.iter().filter(|issue| issue.status == status).count()
                        )</span>
                    </h2>
                    <ul class="list">
                        for issue in issues.iter().filter(|issue| issue.status == status) {
                            <li>
                                <a href=(with_lang(&format!("/issues/{}", issue.id), lang))>
                                    <span class="mono">(issue.identifier.clone())</span>
                                    " " <strong>(issue.title.clone())</strong>
                                </a>
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/issues/{}/status/ui", issue.id), lang))>
                                    <select name="status">
                                        for candidate in moveable {
                                            if candidate != issue.status {
                                                <option value=(candidate)>(candidate)</option>
                                            }
                                        }
                                    </select>
                                    <button type="submit">(t(lang, "board.move"))</button>
                                </form>
                            </li>
                        }
                    </ul>
                </section>
            }
        </div>
    }
}

/// Search query for the search page.
#[topcoat::router::query_params]
struct SearchQuery {
    /// Search term.
    q: Option<String>,
}

/// Search page: company/task search over issues.
#[page("/companies/{company_id}/search")]
pub async fn search(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let query = topcoat::router::query_params::<SearchQuery>(cx)
        .ok()
        .and_then(|params| params.q.clone())
        .unwrap_or_default();
    let trimmed = query.trim().to_owned();
    let results = if trimmed.is_empty() {
        Vec::new()
    } else {
        state
            .issues
            .search(&company_id, &trimmed)
            .await
            .map_err(to_topcoat_error)?
    };
    view! {
        <h1 class="page-title">(t(lang, "search.title"))</h1>
        <form class="inline-form" method="get"
              action=(with_lang(&format!("/companies/{company_id}/search"), lang))>
            <input type="text" name="q" value=(trimmed.clone()) placeholder=(t(lang, "search.placeholder"))>
            <button type="submit">(t(lang, "search.submit"))</button>
        </form>
        if !trimmed.is_empty() {
            if results.is_empty() {
                <p class="empty">(t(lang, "search.noResults"))</p>
            } else {
                <ul class="list">
                    for issue in results {
                        <li>
                            <a href=(with_lang(&format!("/issues/{}", issue.id), lang))>
                                <span class="mono">(issue.identifier.clone())</span>
                                " " <strong>(issue.title.clone())</strong>
                            </a>
                            " " <span class=(status_badge_class(&issue.status))>(issue.status)</span>
                        </li>
                    }
                </ul>
            }
        }
    }
}

/// Settings page: company profile, budget, secrets, and skills.
#[page("/companies/{company_id}/settings")]
pub async fn settings(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company = state
        .companies
        .get(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let Some(company) = company else {
        return Err(topcoat::router::error::not_found().into());
    };
    let secrets = state
        .secrets
        .list_secrets(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let skills = state
        .skills
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "settings.title"))</h1>

        <section>
            <h2>(t(lang, "settings.company"))</h2>
            <form class="stack-form" method="post" action=(with_lang(&format!("/companies/{company_id}/settings/ui"), lang))>
                <input type="hidden" name="action" value="company">
                <label>"Name"</label>
                <input type="text" name="name" value=(company.name)>
                <label>"Description"</label>
                <input type="text" name="description" value=(company.description.as_deref().unwrap_or_default())>
                <button type="submit">(t(lang, "settings.save"))</button>
            </form>
        </section>

        <section>
            <h2>(t(lang, "settings.budget"))</h2>
            <form class="inline-form" method="post" action=(with_lang(&format!("/companies/{company_id}/settings/ui"), lang))>
                <input type="hidden" name="action" value="budget">
                <input type="number" name="budgetMonthlyCents" value=(company.budget_monthly_cents) min="0">
                <button type="submit">(t(lang, "settings.save"))</button>
            </form>
        </section>

        <section>
            <h2>(t(lang, "settings.secrets"))</h2>
            if secrets.is_empty() {
                <p class="empty">(t(lang, "settings.noSecrets"))</p>
            } else {
                <ul class="list">
                    for secret in secrets {
                        <li>
                            <span class="mono">(secret.name)</span>
                            " " <span class="badge badge-default">(secret.latest_version) "v"</span>
                        </li>
                    }
                </ul>
            }
            <form class="inline-form" method="post" action=(with_lang(&format!("/companies/{company_id}/settings/ui"), lang))>
                <input type="hidden" name="action" value="secret">
                <input type="text" name="name" placeholder=(t(lang, "settings.secretName"))>
                <input type="password" name="value" placeholder=(t(lang, "settings.secretValue"))>
                <button type="submit">(t(lang, "settings.add"))</button>
            </form>
        </section>

        <section>
            <h2>(t(lang, "settings.skills"))</h2>
            if skills.is_empty() {
                <p class="empty">(t(lang, "settings.noSkills"))</p>
            } else {
                <ul class="list">
                    for skill in skills {
                        <li>
                            <strong>(skill.name)</strong>
                            " " <span class="badge badge-default">(skill.status)</span>
                            if let Some(description) = &skill.description {
                                " " <span class="meta-row">(description)</span>
                            }
                        </li>
                    }
                </ul>
            }
            <form class="inline-form" method="post" action=(with_lang(&format!("/companies/{company_id}/settings/ui"), lang))>
                <input type="hidden" name="action" value="skill">
                <input type="text" name="name" placeholder=(t(lang, "settings.skillName"))>
                <input type="text" name="description" placeholder=(t(lang, "settings.skillDescription"))>
                <button type="submit">(t(lang, "settings.add"))</button>
            </form>
        </section>
    }
}

/// Shared `{id}` path parameter for UI pages.
#[path_param(error = bad_request("Invalid id"))]
pub(crate) struct Id(String);
