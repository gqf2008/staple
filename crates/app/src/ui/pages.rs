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
            <a href=(with_lang(&format!("/companies/{company_id}/dashboard"), lang))>(t(lang, "dashboard.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/dashboard/live"), lang))>(t(lang, "live.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/org-chart"), lang))>(t(lang, "orgChart.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/board"), lang))>(t(lang, "nav.board"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/issues"), lang))>(t(lang, "nav.issues"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/search"), lang))>(t(lang, "nav.search"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/agents"), lang))>(t(lang, "agents.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/inbox"), lang))>(t(lang, "inbox.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/decision-desk"), lang))>(t(lang, "decision.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/approvals"), lang))>(t(lang, "nav.approvals"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/activity"), lang))>(t(lang, "nav.activity"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/cases"), lang))>(t(lang, "cases.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/access"), lang))>(t(lang, "access.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/costs"), lang))>(t(lang, "costs.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/routines"), lang))>(t(lang, "routines.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/secrets"), lang))>(t(lang, "settings.secrets"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/skills"), lang))>(t(lang, "settings.skills"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/workspaces"), lang))>(t(lang, "workspaces.title"))</a>
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

/// Agents list page.
#[page("/companies/{company_id}/agents")]
pub async fn agents(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let agents = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "agents.title"))</h1>
        if agents.is_empty() {
            <p class="empty">(t(lang, "agents.noAgents"))</p>
        } else {
            <ul class="list">
                for agent in agents {
                    <li>
                        <a href=(with_lang(&format!("/agents/{}", agent.id), lang))>
                            <strong>(agent.name)</strong>
                        </a>
                        " " <span class=(status_badge_class(&agent.status))>(agent.status)</span>
                        " " <span class="badge badge-default">(agent.role)</span>
                        " " <span class="meta-row">(agent.adapter_type)</span>
                    </li>
                }
            </ul>
        }
    }
}

/// Agent detail page: profile, runtime state, sessions, wakeups, and controls.
#[page("/agents/{agent_id}")]
pub async fn agent_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let agent_id = path_param::<AgentPathId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .agents
        .company_of(&agent_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let Some(agent) = state
        .agents
        .get(&company_id, &agent_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let runtime = state
        .agent_runtime
        .runtime_get(&company_id, &agent_id)
        .await
        .map_err(to_topcoat_error)?;
    let sessions = state
        .agent_runtime
        .session_list(&company_id)
        .await
        .map_err(to_topcoat_error)?
        .into_iter()
        .filter(|s| s.agent_id == agent_id)
        .collect::<Vec<_>>();
    let wakeups = state
        .agent_runtime
        .wakeup_list(&company_id)
        .await
        .map_err(to_topcoat_error)?
        .into_iter()
        .filter(|w| w.agent_id == agent_id)
        .collect::<Vec<_>>();
    let status_url = with_lang(&format!("/agents/{agent_id}/status/ui"), lang);
    view! {
        <h1 class="page-title">(agent.name)</h1>
        <p class="mono">(agent.id)</p>
        <p>
            <span class=(status_badge_class(&agent.status))>(agent.status.clone())</span>
            " " <span class="badge badge-default">(agent.role.clone())</span>
            " " <span class="badge badge-default">(agent.adapter_type.clone())</span>
        </p>
        if agent.status == "active" {
            <form class="inline-form" method="post" action=(status_url)>
                <input type="hidden" name="status" value="paused">
                <input type="text" name="pause_reason" placeholder=(t(lang, "agent.pauseReason"))>
                <button type="submit" class="secondary">(t(lang, "agent.pause"))</button>
            </form>
        } else {
            <form class="inline-form" method="post" action=(status_url)>
                <input type="hidden" name="status" value="active">
                <button type="submit">(t(lang, "agent.resume"))</button>
            </form>
        }
        <section>
            <h2>(t(lang, "agent.runtime"))</h2>
            if let Some(runtime) = &runtime {
                <ul class="list">
                    <li>(t(lang, "agent.session")) ": " <span class="mono">(runtime.session_id.clone().unwrap_or_default())</span></li>
                    <li>(t(lang, "agent.lastRunStatus")) ": " <span class=(status_badge_class(runtime.last_run_status.as_deref().unwrap_or("backlog")))>(runtime.last_run_status.clone().unwrap_or_default())</span></li>
                    <li>(t(lang, "agent.tokens")) ": " (runtime.total_input_tokens) " / " (runtime.total_output_tokens)</li>
                    <li>(t(lang, "agent.cost")) ": " (runtime.total_cost_cents) "¢"</li>
                </ul>
            } else {
                <p class="empty">(t(lang, "agent.noRuntime"))</p>
            }
        </section>
        <section>
            <h2>(t(lang, "agent.sessions"))</h2>
            if sessions.is_empty() {
                <p class="empty">(t(lang, "agent.noSessions"))</p>
            } else {
                <ul class="list">
                    for session in sessions {
                        <li>
                            <span class="mono">(session.task_key)</span>
                            " " <span class="meta-row">(session.adapter_type)</span>
                            if let Some(display) = &session.session_display_id {
                                " " <span class="mono">(display.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "agent.wakeups"))</h2>
            if wakeups.is_empty() {
                <p class="empty">(t(lang, "agent.noWakeups"))</p>
            } else {
                <ul class="list">
                    for wakeup in wakeups {
                        <li>
                            <span class="mono">(wakeup.source)</span>
                            " " <span class=(status_badge_class(&wakeup.status))>(wakeup.status)</span>
                            " " <span class="meta-row">(wakeup.requested_at)</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "agent.budget"))</h2>
            <p>(t(lang, "agent.monthlyBudget")) ": " (agent.budget_monthly_cents) "¢"
               " / " (t(lang, "agent.spent")) ": " (agent.spent_monthly_cents) "¢"</p>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/agents/{agent_id}/budget/ui"), lang))>
                <input type="number" name="budgetMonthlyCents" value=(agent.budget_monthly_cents) min="0">
                <button type="submit">(t(lang, "settings.save"))</button>
            </form>
        </section>
    }
}

/// Decision desk page: queues, items, triage, retention.
#[page("/companies/{company_id}/decision-desk")]
pub async fn decision_desk(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let queues = state
        .decisions
        .list_queues(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let triage = state
        .decisions
        .list_triage(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let retention = state
        .decisions
        .list_retention(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let outbox = state
        .decisions
        .list_outbox(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "decision.title"))</h1>
        <section>
            <h2>(t(lang, "decision.queues"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/decision-queues/ui"), lang))>
                <input type="text" name="name" placeholder=(t(lang, "decision.queueName"))>
                <button type="submit">(t(lang, "settings.add"))</button>
            </form>
            if queues.is_empty() {
                <p class="empty">(t(lang, "decision.noQueues"))</p>
            } else {
                <ul class="list">
                    for queue in queues {
                        <li>
                            <strong>(queue.name)</strong>
                            " " <span class="mono">(queue.id)</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "decision.triage"))</h2>
            if triage.is_empty() {
                <p class="empty">(t(lang, "decision.noTriage"))</p>
            } else {
                <ul class="list">
                    for row in triage {
                        <li>
                            <span class="mono">(row.source_kind.clone()) ":" (row.source_id.clone())</span>
                            " " <span class=(status_badge_class(row.decision.as_deref().unwrap_or("backlog")))>(row.decision.clone().unwrap_or_default())</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "decision.retention"))</h2>
            if retention.is_empty() {
                <p class="empty">(t(lang, "decision.noRetention"))</p>
            } else {
                <ul class="list">
                    for row in retention {
                        <li>
                            <span class="mono">(row.source_kind.clone()) ":" (row.source_id.clone())</span>
                            " keep=" (row.keep) " archived=" (row.archived)
                            if row.archived {
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/companies/{company_id}/decision-retention/{}/{}/restore/ui", row.source_kind, row.source_id), lang))>
                                    <button type="submit" class="secondary">(t(lang, "decision.restore"))</button>
                                </form>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "decision.outbox"))</h2>
            if outbox.is_empty() {
                <p class="empty">(t(lang, "decision.noOutbox"))</p>
            } else {
                <ul class="list">
                    for row in outbox {
                        <li>
                            <span class="mono">(row.dedupe_key)</span>
                            " " <span class=(status_badge_class(&row.status))>(row.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Inbox page: unarchived issues with archive/restore controls.
#[page("/companies/{company_id}/inbox")]
pub async fn inbox(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let issues = state
        .issues
        .list_inbox(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "inbox.title"))</h1>
        if issues.is_empty() {
            <p class="empty">(t(lang, "inbox.empty"))</p>
        } else {
            <ul class="list">
                for issue in issues {
                    <li>
                        <a href=(with_lang(&format!("/issues/{}", issue.id), lang))>
                            <span class="mono">(issue.identifier.clone())</span>
                            " " <strong>(issue.title.clone())</strong>
                        </a>
                        " " <span class=(status_badge_class(&issue.status))>(issue.status)</span>
                        <form class="inline-form" method="post"
                              action=(with_lang(&format!("/issues/{}/archive/ui", issue.id), lang))>
                            <button type="submit" class="secondary">(t(lang, "inbox.archive"))</button>
                        </form>
                    </li>
                }
            </ul>
        }
    }
}

/// Company access page: memberships, invites, join requests, permission grants.
#[page("/companies/{company_id}/access")]
pub async fn access(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let members = state
        .memberships
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let invites = state
        .invites
        .list_invites(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let joins = state
        .invites
        .list_join_requests(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let grants = state
        .permission_grants
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "access.title"))</h1>
        <section>
            <h2>(t(lang, "access.members"))</h2>
            if members.is_empty() {
                <p class="empty">(t(lang, "access.noMembers"))</p>
            } else {
                <ul class="list">
                    for member in members {
                        <li>
                            <span class="mono">(member.principal_type) ":" (member.principal_id)</span>
                            " " <span class="badge badge-default">(member.status)</span>
                            if let Some(role) = &member.membership_role {
                                " " <span class="badge badge-default">(role.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "access.invites"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/invites/ui"), lang))>
                <input type="text" name="name" placeholder=(t(lang, "access.inviteName"))>
                <button type="submit">(t(lang, "access.invite"))</button>
            </form>
            if invites.is_empty() {
                <p class="empty">(t(lang, "access.noInvites"))</p>
            } else {
                <ul class="list">
                    for invite in invites {
                        <li>
                            <span class="mono">(invite.invite_type)</span>
                            " " <span class="meta-row">(invite.expires_at)</span>
                            <form class="inline-form" method="post"
                                  action=(with_lang(&format!("/companies/{company_id}/invites/{}/revoke/ui", invite.id), lang))>
                                <button type="submit" class="destructive">(t(lang, "access.revoke"))</button>
                            </form>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "access.joinRequests"))</h2>
            if joins.is_empty() {
                <p class="empty">(t(lang, "access.noJoinRequests"))</p>
            } else {
                <ul class="list">
                    for join in joins {
                        <li>
                            <span class="mono">(join.request_type)</span>
                            " " <span class=(status_badge_class(&join.status))>(join.status.clone())</span>
                            if join.status == "pending_approval" {
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/companies/{company_id}/join-requests/{}/approve/ui", join.id), lang))>
                                    <button type="submit">(t(lang, "access.approve"))</button>
                                </form>
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/companies/{company_id}/join-requests/{}/reject/ui", join.id), lang))>
                                    <button type="submit" class="destructive">(t(lang, "access.reject"))</button>
                                </form>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "access.grants"))</h2>
            if grants.is_empty() {
                <p class="empty">(t(lang, "access.noGrants"))</p>
            } else {
                <ul class="list">
                    for grant in grants {
                        <li>
                            <span class="mono">(grant.principal_type) ":" (grant.principal_id)</span>
                            " " <strong>(grant.permission_key)</strong>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Costs page: budget summary and per-agent spending.
#[page("/companies/{company_id}/costs")]
pub async fn costs(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let summary = state
        .costs
        .summary(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let rows = state
        .costs
        .by_agent(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "costs.title"))</h1>
        if let Some(summary) = &summary {
            <section>
                <h2>(t(lang, "costs.summary"))</h2>
                <ul class="list">
                    <li>(t(lang, "costs.budget")) ": " (summary.budget_monthly_cents) "¢"</li>
                    <li>(t(lang, "costs.spent")) ": " (summary.spent_monthly_cents) "¢"</li>
                    <li>(t(lang, "costs.pausedAgents")) ": " (summary.paused_agents)</li>
                </ul>
            </section>
        }
        <section>
            <h2>(t(lang, "costs.byAgent"))</h2>
            if rows.is_empty() {
                <p class="empty">(t(lang, "costs.noRows"))</p>
            } else {
                <ul class="list">
                    for row in rows {
                        <li>
                            <span class="mono">(row.agent_id)</span>
                            " " <span class="meta-row">(t(lang, "costs.spent")) ": " (row.spent_monthly_cents) "¢ / " (row.budget_monthly_cents) "¢"</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Routines page: list + create + trigger.
#[page("/companies/{company_id}/routines")]
pub async fn routines(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let routines = state
        .routines
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "routines.title"))</h1>
        <form class="inline-form" method="post"
              action=(with_lang(&format!("/companies/{company_id}/routines/ui"), lang))>
            <input type="text" name="title" placeholder=(t(lang, "routines.title"))>
            <button type="submit">(t(lang, "settings.add"))</button>
        </form>
        if routines.is_empty() {
            <p class="empty">(t(lang, "routines.noRoutines"))</p>
        } else {
            <ul class="list">
                for routine in routines {
                    <li>
                        <strong>(routine.title)</strong>
                        " " <span class=(status_badge_class(&routine.status))>(routine.status)</span>
                        " " <span class="meta-row">(t(lang, "routines.rev")) " " (routine.latest_revision_number)</span>
                        <form class="inline-form" method="post"
                              action=(with_lang(&format!("/routines/{}/trigger/ui", routine.id), lang))>
                            <button type="submit">(t(lang, "routines.trigger"))</button>
                        </form>
                    </li>
                }
            </ul>
        }
    }
}

/// Secrets page: list + create.
#[page("/companies/{company_id}/secrets")]
pub async fn secrets_page(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let secrets = state
        .secrets
        .list_secrets(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "settings.secrets"))</h1>
        <form class="inline-form" method="post"
              action=(with_lang(&format!("/companies/{company_id}/secrets/ui"), lang))>
            <input type="text" name="name" placeholder=(t(lang, "settings.secretName"))>
            <input type="password" name="value" placeholder=(t(lang, "settings.secretValue"))>
            <button type="submit">(t(lang, "settings.add"))</button>
        </form>
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
    }
}

/// Skills page: list + create.
#[page("/companies/{company_id}/skills")]
pub async fn skills_page(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let skills = state
        .skills
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "settings.skills"))</h1>
        <form class="inline-form" method="post"
              action=(with_lang(&format!("/companies/{company_id}/skills/ui"), lang))>
            <input type="text" name="name" placeholder=(t(lang, "settings.skillName"))>
            <input type="text" name="description" placeholder=(t(lang, "settings.skillDescription"))>
            <button type="submit">(t(lang, "settings.add"))</button>
        </form>
        if skills.is_empty() {
            <p class="empty">(t(lang, "settings.noSkills"))</p>
        } else {
            <ul class="list">
                for skill in skills {
                    <li>
                        <strong>(skill.name)</strong>
                        " " <span class="badge badge-default">(skill.status)</span>
                        if let Some(description) = &skill.description {
                            " " <span class="meta-row">(description.clone())</span>
                        }
                    </li>
                }
            </ul>
        }
    }
}

/// Instance settings page: instance roles, board API keys, CLI challenges.
#[page("/instance/settings")]
pub async fn instance_settings(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let state = app_context::<AppState>(cx);
    let roles = state
        .memberships
        .list_roles()
        .await
        .map_err(to_topcoat_error)?;
    let keys = state
        .board_keys
        .list_keys()
        .await
        .map_err(to_topcoat_error)?;
    let challenges = state
        .board_keys
        .list_challenges()
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "instance.title"))</h1>
        <section>
            <h2>(t(lang, "instance.roles"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang("/instance/user-roles/ui", lang))>
                <input type="text" name="userId" placeholder=(t(lang, "instance.userId"))>
                <button type="submit">(t(lang, "settings.add"))</button>
            </form>
            if roles.is_empty() {
                <p class="empty">(t(lang, "instance.noRoles"))</p>
            } else {
                <ul class="list">
                    for role in roles {
                        <li>
                            <span class="mono">(role.user_id)</span>
                            " " <span class="badge badge-default">(role.role)</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "instance.boardKeys"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang("/board-api-keys/ui", lang))>
                <input type="text" name="userId" placeholder=(t(lang, "instance.userId"))>
                <input type="text" name="name" placeholder=(t(lang, "instance.keyName"))>
                <button type="submit">(t(lang, "settings.add"))</button>
            </form>
            if keys.is_empty() {
                <p class="empty">(t(lang, "instance.noKeys"))</p>
            } else {
                <ul class="list">
                    for key in keys {
                        <li>
                            <span class="mono">(key.name)</span>
                            " " <span class="meta-row">(key.user_id)</span>
                            " " <span class=(status_badge_class(if key.revoked_at.is_some() { "cancelled" } else { "active" }))>
                                (if key.revoked_at.is_some() { "revoked" } else { "active" })
                            </span>
                            if key.revoked_at.is_none() {
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/board-api-keys/{}/revoke/ui", key.id), lang))>
                                    <button type="submit" class="destructive">(t(lang, "access.revoke"))</button>
                                </form>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "instance.challenges"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang("/cli-auth-challenges/ui", lang))>
                <input type="text" name="command" placeholder=(t(lang, "instance.challengeCommand"))>
                <input type="text" name="pendingKeyName" placeholder=(t(lang, "instance.keyName"))>
                <button type="submit">(t(lang, "instance.challenge"))</button>
            </form>
            if challenges.is_empty() {
                <p class="empty">(t(lang, "instance.noChallenges"))</p>
            } else {
                <ul class="list">
                    for challenge in challenges {
                        <li>
                            <span class="mono">(challenge.pending_key_name)</span>
                            " " <span class=(status_badge_class(if challenge.approved_at.is_some() { "done" } else if challenge.cancelled_at.is_some() { "cancelled" } else { "active" }))>
                                (if challenge.approved_at.is_some() { "approved" } else if challenge.cancelled_at.is_some() { "cancelled" } else { "pending" })
                            </span>
                            if challenge.approved_at.is_none() && challenge.cancelled_at.is_none() {
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/cli-auth-challenges/{}/approve/ui", challenge.id), lang))>
                                    <button type="submit">(t(lang, "access.approve"))</button>
                                </form>
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/cli-auth-challenges/{}/cancel/ui", challenge.id), lang))>
                                    <button type="submit" class="destructive">(t(lang, "access.reject"))</button>
                                </form>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// `{agent_id}` path parameter for UI pages.
#[path_param(error = bad_request("Invalid agent id"))]
pub(crate) struct AgentPathId(String);

/// Dashboard: issue/agent/budget statistics plus recent activity.
#[page("/companies/{company_id}/dashboard")]
pub async fn dashboard(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let issues = state
        .issues
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let summary = state
        .costs
        .summary(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let activity_rows = state
        .activity
        .list(&company_id, 10)
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
    view! {
        <h1 class="page-title">(t(lang, "dashboard.title"))</h1>
        <section>
            <h2>(t(lang, "dashboard.issues"))</h2>
            <ul class="list">
                for status in statuses {
                    <li>
                        <span class=(status_badge_class(status))>(status)</span>
                        " " <strong>(issues.iter().filter(|i| i.status == status).count())</strong>
                    </li>
                }
                <li>
                    <span class="badge badge-default">(t(lang, "dashboard.total"))</span>
                    " " <strong>(issues.len())</strong>
                </li>
            </ul>
        </section>
        <section>
            <h2>(t(lang, "dashboard.agents"))</h2>
            if agent_rows.is_empty() {
                <p class="empty">(t(lang, "agents.noAgents"))</p>
            } else {
                <ul class="list">
                    for agent in agent_rows {
                        <li>
                            <a href=(with_lang(&format!("/agents/{}", agent.id), lang))>
                                <strong>(agent.name)</strong>
                            </a>
                            " " <span class=(status_badge_class(&agent.status))>(agent.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>
        if let Some(summary) = &summary {
            <section>
                <h2>(t(lang, "dashboard.budget"))</h2>
                <ul class="list">
                    <li>(t(lang, "costs.budget")) ": " (summary.budget_monthly_cents) "¢"</li>
                    <li>(t(lang, "costs.spent")) ": " (summary.spent_monthly_cents) "¢"</li>
                    <li>(t(lang, "costs.pausedAgents")) ": " (summary.paused_agents)</li>
                </ul>
            </section>
        }
        <section>
            <h2>(t(lang, "dashboard.activity"))</h2>
            if activity_rows.is_empty() {
                <p class="empty">(t(lang, "activity.noActivity"))</p>
            } else {
                <ul class="list">
                    for entry in activity_rows {
                        <li>
                            <span class="mono">(entry.created_at)</span>
                            " " <strong>(entry.action)</strong>
                            " " <span class="meta-row">(entry.actor_type) "/" (entry.actor_id)</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Project detail: attributes, linked issues, and an edit form.
#[page("/projects/{project_id}")]
pub async fn project_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let project_id = path_param::<ProjectId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(project) = state
        .projects
        .get(&project_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let issues = state
        .issues
        .list(&project.company_id)
        .await
        .map_err(to_topcoat_error)?
        .into_iter()
        .filter(|issue| issue.project_id.as_deref() == Some(project_id.as_str()))
        .collect::<Vec<_>>();
    let statuses = [
        "backlog",
        "planned",
        "in_progress",
        "completed",
        "cancelled",
    ];
    view! {
        <h1 class="page-title">(project.name.clone())</h1>
        <p class="mono">(project.id.clone())</p>
        <p>
            <span class=(status_badge_class(&project.status))>(project.status.clone())</span>
            if let Some(description) = &project.description {
                " " <span class="meta-row">(description.clone())</span>
            }
        </p>
        <section>
            <h2>(t(lang, "projects.edit"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/projects/{project_id}/edit/ui"), lang))>
                <label>"Name"</label>
                <input type="text" name="name" value=(project.name.clone())>
                <label>"Description"</label>
                <input type="text" name="description" value=(project.description.clone().unwrap_or_default())>
                <label>"Status"</label>
                <select name="status">
                    for status in statuses {
                        if status == project.status {
                            <option value=(status) selected="selected">(status)</option>
                        } else {
                            <option value=(status)>(status)</option>
                        }
                    }
                </select>
                <button type="submit">(t(lang, "settings.save"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "projects.issues"))</h2>
            if issues.is_empty() {
                <p class="empty">(t(lang, "empty.noIssues"))</p>
            } else {
                <ul class="list">
                    for issue in issues {
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
        </section>
    }
}

/// Workspaces page: project/execution workspaces, runtime services, operations.
#[page("/companies/{company_id}/workspaces")]
pub async fn workspaces(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let project_workspaces = state
        .workspaces
        .list_project_workspaces(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;
    let execution_workspaces = state
        .workspaces
        .list_execution_workspaces(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;
    let services = state
        .workspaces
        .list_runtime_services(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let operations = state
        .workspaces
        .list_operations(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "workspaces.title"))</h1>
        <section>
            <h2>(t(lang, "workspaces.project"))</h2>
            if project_workspaces.is_empty() {
                <p class="empty">(t(lang, "workspaces.empty"))</p>
            } else {
                <ul class="list">
                    for workspace in project_workspaces {
                        <li>
                            <strong>(workspace.name)</strong>
                            " " <span class="badge badge-default">(workspace.source_type)</span>
                            if let Some(repo) = &workspace.repo_url {
                                " " <span class="mono">(repo.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "workspaces.execution"))</h2>
            if execution_workspaces.is_empty() {
                <p class="empty">(t(lang, "workspaces.empty"))</p>
            } else {
                <ul class="list">
                    for workspace in execution_workspaces {
                        <li>
                            <span class="mono">(workspace.name)</span>
                            " " <span class=(status_badge_class(if workspace.materialized { "done" } else { "backlog" }))>
                                (if workspace.materialized { "materialized" } else { "not materialized" })
                            </span>
                            if !workspace.materialized {
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/companies/{company_id}/workspaces/{}/materialize/ui", workspace.id), lang))>
                                    <button type="submit" class="secondary">(t(lang, "workspaces.materialize"))</button>
                                </form>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "workspaces.services"))</h2>
            if services.is_empty() {
                <p class="empty">(t(lang, "workspaces.empty"))</p>
            } else {
                <ul class="list">
                    for service in services {
                        <li>
                            <strong>(service.service_name)</strong>
                            " " <span class="badge badge-default">(service.status)</span>
                            " " <span class="mono">(service.scope_type)</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "workspaces.operations"))</h2>
            if operations.is_empty() {
                <p class="empty">(t(lang, "workspaces.empty"))</p>
            } else {
                <ul class="list">
                    for operation in operations {
                        <li>
                            <span class=(status_badge_class(&operation.phase))>(operation.phase)</span>
                            " " <span class="mono">(operation.command.clone().unwrap_or_default())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Adapters page: registered adapters and plugin diagnostics.
#[page("/adapters")]
pub async fn adapters(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let state = app_context::<AppState>(cx);
    let names = state.adapters.names();
    let reports = state.plugin_reports.clone();
    view! {
        <h1 class="page-title">(t(lang, "adapters.title"))</h1>
        <section>
            <h2>(t(lang, "adapters.registered"))</h2>
            if names.is_empty() {
                <p class="empty">(t(lang, "adapters.empty"))</p>
            } else {
                <ul class="list">
                    for name in names {
                        <li>
                            <a href=(with_lang(&format!("/adapters/{name}"), lang))>
                                <span class="mono">(name)</span>
                            </a>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "adapters.plugins"))</h2>
            if reports.is_empty() {
                <p class="empty">(t(lang, "adapters.noPlugins"))</p>
            } else {
                <ul class="list">
                    for report in reports {
                        <li>
                            <span class="mono">(report.r#type)</span>
                            " " <span class=(status_badge_class(if report.loaded { "done" } else { "cancelled" }))>
                                (if report.loaded { "loaded" } else { "failed" })
                            </span>
                            if let Some(error) = &report.error {
                                " " <span class="meta-row">(error.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Org chart: agents rendered as a `reports_to` tree.
#[page("/companies/{company_id}/org-chart")]
pub async fn org_chart(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    // Depth-first flatten of the reports_to tree.
    let mut by_parent: std::collections::HashMap<Option<String>, Vec<&staple_data::AgentRecord>> =
        std::collections::HashMap::new();
    for agent in &agent_rows {
        by_parent
            .entry(agent.reports_to.clone())
            .or_default()
            .push(agent);
    }
    fn walk(
        by_parent: &std::collections::HashMap<Option<String>, Vec<&staple_data::AgentRecord>>,
        flat: &mut Vec<(usize, String, String, String, String)>,
        parent: Option<String>,
        depth: usize,
    ) {
        let Some(children) = by_parent.get(&parent) else {
            return;
        };
        for agent in children {
            flat.push((
                depth,
                agent.id.clone(),
                agent.name.clone(),
                agent.status.clone(),
                agent.role.clone(),
            ));
            walk(by_parent, flat, Some(agent.id.clone()), depth + 1);
        }
    }
    let mut flat: Vec<(usize, String, String, String, String)> = Vec::new();
    walk(&by_parent, &mut flat, None, 0);
    let indent = |depth: usize| -> String { "\u{3000}".repeat(depth) };
    view! {
        <h1 class="page-title">(t(lang, "orgChart.title"))</h1>
        if flat.is_empty() {
            <p class="empty">(t(lang, "agents.noAgents"))</p>
        } else {
            <ul class="list">
                for (depth, id, name, status, role) in flat {
                    <li>
                        <span class="mono">(indent(depth))</span>
                        <a href=(with_lang(&format!("/agents/{id}"), lang))>
                            <strong>(name)</strong>
                        </a>
                        " " <span class=(status_badge_class(&status))>(status)</span>
                        " " <span class="badge badge-default">(role)</span>
                    </li>
                }
            </ul>
        }
    }
}

/// Adapter detail: invoke, observe, and cancel a run.
#[page("/adapters/{type}")]
pub async fn adapter_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let adapter_type = path_param::<Type>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(adapter) = state.adapters.get(&adapter_type) else {
        return Err(topcoat::router::error::not_found().into());
    };
    let run_id = topcoat::router::query_params::<AdapterRunQuery>(cx)
        .ok()
        .and_then(|query| query.run_id.clone());
    let status = match run_id.as_deref() {
        Some(id) => match adapter.observe(id).await {
            Ok(observed) => serde_json::to_string(&observed).ok(),
            Err(_) => None,
        },
        None => None,
    };
    let invoke_url = with_lang(&format!("/adapters/{adapter_type}/invoke/ui"), lang);
    view! {
        <h1 class="page-title">(t(lang, "adapters.detail")) " " <span class="mono">(adapter_type.clone())</span></h1>
        <section>
            <h2>(t(lang, "adapters.invoke"))</h2>
            <form class="stack-form" method="post" action=(invoke_url)>
                <label>(t(lang, "adapters.task"))</label>
                <textarea name="task" rows="4" cols="60"></textarea>
                <button type="submit">(t(lang, "adapters.invoke"))</button>
            </form>
        </section>
        if let Some(run_id) = &run_id {
            <section>
                <h2>(t(lang, "adapters.run")) " " <span class="mono">(run_id.clone())</span></h2>
                if let Some(status) = &status {
                    <p class="mono">(status.clone())</p>
                } else {
                    <p class="empty">(t(lang, "adapters.runUnknown"))</p>
                }
                <form class="inline-form" method="post"
                      action=(with_lang(&format!("/adapters/{adapter_type}/runs/{run_id}/cancel/ui"), lang))>
                    <button type="submit" class="destructive">(t(lang, "adapters.cancel"))</button>
                </form>
            </section>
        }
    }
}

/// Live dashboard: running/recent heartbeat runs plus agent status.
#[page("/companies/{company_id}/dashboard/live")]
pub async fn dashboard_live(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let runs = state
        .heartbeat
        .list(&company_id, None, 50)
        .await
        .map_err(to_topcoat_error)?;
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "live.title"))</h1>
        <section>
            <h2>(t(lang, "live.runs"))</h2>
            if runs.is_empty() {
                <p class="empty">(t(lang, "live.noRuns"))</p>
            } else {
                <ul class="list">
                    for run in runs {
                        <li>
                            <span class="mono">(run.id)</span>
                            " " <span class=(status_badge_class(&run.status))>(run.status)</span>
                            " " <span class="meta-row">(run.invocation_source) " / " (run.agent_id)</span>
                            if let Some(started) = &run.started_at {
                                " " <span class="meta-row">(started.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "live.agents"))</h2>
            if agent_rows.is_empty() {
                <p class="empty">(t(lang, "agents.noAgents"))</p>
            } else {
                <ul class="list">
                    for agent in agent_rows {
                        <li>
                            <a href=(with_lang(&format!("/agents/{}", agent.id), lang))>
                                <strong>(agent.name)</strong>
                            </a>
                            " " <span class=(status_badge_class(&agent.status))>(agent.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Query for the adapter detail page.
#[topcoat::router::query_params]
struct AdapterRunQuery {
    /// Optional run id to observe.
    #[serde(rename = "runId")]
    run_id: Option<String>,
}

/// Cases list page: create form + list.
#[page("/companies/{company_id}/cases")]
pub async fn cases_list(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let rows = state
        .cases
        .list(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "cases.title"))</h1>
        <form class="inline-form" method="post"
              action=(with_lang(&format!("/companies/{company_id}/cases/ui"), lang))>
            <input type="text" name="case_type" placeholder=(t(lang, "cases.type"))>
            <input type="text" name="title" placeholder=(t(lang, "cases.title"))>
            <button type="submit">(t(lang, "settings.add"))</button>
        </form>
        if rows.is_empty() {
            <p class="empty">(t(lang, "cases.empty"))</p>
        } else {
            <ul class="list">
                for case in rows {
                    <li>
                        <a href=(with_lang(&format!("/cases/{}", case.id), lang))>
                            <span class="mono">(case.identifier.clone())</span>
                            " " <strong>(case.title.clone())</strong>
                        </a>
                        " " <span class=(status_badge_class(&case.status))>(case.status)</span>
                        " " <span class="badge badge-default">(case.case_type)</span>
                    </li>
                }
            </ul>
        }
    }
}

/// Case detail: attributes, fields, parent, and status moves.
#[page("/cases/{case_id}")]
pub async fn case_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let case_id = path_param::<CasePathId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .cases
        .company_of(&case_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let Some(case) = state
        .cases
        .get(&company_id, &case_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let next_statuses = ["in_progress", "in_review", "approved", "done", "cancelled"];
    let status_url = with_lang(&format!("/cases/{case_id}/status/ui"), lang);
    view! {
        <h1 class="page-title">(case.title.clone())</h1>
        <p class="mono">(case.identifier.clone()) " #" (case.case_number)</p>
        <p>
            <span class=(status_badge_class(&case.status))>(case.status.clone())</span>
            " " <span class="badge badge-default">(case.case_type)</span>
            if let Some(summary) = &case.summary {
                " " <span class="meta-row">(summary.clone())</span>
            }
        </p>
        <section>
            <h2>(t(lang, "cases.move"))</h2>
            <form class="inline-form" method="post" action=(status_url)>
                <select name="status">
                    for candidate in next_statuses {
                        if candidate != case.status {
                            <option value=(candidate)>(candidate)</option>
                        }
                    }
                </select>
                <button type="submit">(t(lang, "cases.move"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "cases.fields"))</h2>
            <p class="mono">(case.fields.to_string())</p>
        </section>
        if let Some(parent_id) = &case.parent_case_id {
            <section>
                <h2>(t(lang, "cases.parent"))</h2>
                <a href=(with_lang(&format!("/cases/{parent_id}"), lang))>
                    <span class="mono">(parent_id.clone())</span>
                </a>
            </section>
        }
    }
}

/// `{case_id}` path parameter for UI pages.
#[path_param(error = bad_request("Invalid case id"))]
pub(crate) struct CasePathId(String);

/// `{type}` path parameter for UI pages.
#[path_param(error = bad_request("Invalid adapter type"))]
pub(crate) struct Type(String);

/// `{project_id}` path parameter for UI pages.
#[path_param(error = bad_request("Invalid project id"))]
pub(crate) struct ProjectId(String);

/// Shared `{id}` path parameter for UI pages.
#[path_param(error = bad_request("Invalid id"))]
pub(crate) struct Id(String);
