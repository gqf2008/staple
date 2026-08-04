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

/// Typed `{goal_id}` path segment for UI pages.
#[path_param(error = bad_request("Invalid goal id"))]
pub(crate) struct GoalId(String);

/// Typed `{decision_id}` path segment for UI pages.
#[path_param(error = bad_request("Invalid decision id"))]
pub(crate) struct DecisionId(String);

/// Typed `{skill_id}` path segment for UI pages.
#[path_param(error = bad_request("Invalid skill id"))]
pub(crate) struct SkillId(String);

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
    let goal_rows = state
        .goals
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let project_rows = state
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
            <a href=(with_lang(&format!("/companies/{company_id}/decisions"), lang))>(t(lang, "decisions.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/decision-training-examples"), lang))>(t(lang, "training.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/status-cards"), lang))>(t(lang, "statusCards.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/summary-slots"), lang))>(t(lang, "summarySlots.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/finance-events"), lang))>(t(lang, "finance.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/feedback-votes"), lang))>(t(lang, "feedback.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/secret-bindings"), lang))>(t(lang, "secretBindings.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/user-secrets"), lang))>(t(lang, "userSecrets.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/approvals"), lang))>(t(lang, "nav.approvals"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/activity"), lang))>(t(lang, "nav.activity"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/cases"), lang))>(t(lang, "cases.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/pipelines"), lang))>(t(lang, "pipelines.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/access"), lang))>(t(lang, "access.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/costs"), lang))>(t(lang, "costs.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/routines"), lang))>(t(lang, "routines.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/goals"), lang))>(t(lang, "nav.goals"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/projects"), lang))>(t(lang, "nav.projects"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/secrets"), lang))>(t(lang, "settings.secrets"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/skills"), lang))>(t(lang, "settings.skills"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/workspaces"), lang))>(t(lang, "workspaces.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/settings"), lang))>(t(lang, "nav.settings"))</a>
        </nav>

        <section>
            <h2>(t(lang, "section.goals"))</h2>
            if goal_rows.is_empty() {
                <p class="empty">(t(lang, "empty.noGoals"))</p>
            } else {
                <ul class="list">
                    for goal in goal_rows {
                        <li>
                            <a href=(with_lang(&format!("/goals/{}", goal.id), lang))>
                                <strong>(goal.title)</strong>
                            </a>
                            " " <span class="badge badge-default">(goal.level)</span>
                            " " <span class=(status_badge_class(&goal.status))>(goal.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>(t(lang, "section.projects"))</h2>
            if project_rows.is_empty() {
                <p class="empty">(t(lang, "empty.noProjects"))</p>
            } else {
                <ul class="list">
                    for project in project_rows {
                        <li>
                            <a href=(with_lang(&format!("/projects/{}", project.id), lang))>
                                <strong>(project.name)</strong>
                            </a>
                            " " <span class=(status_badge_class(&project.status))>(project.status)</span>
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
                <section class="board-column" data-status=(status)>
                    <h2 class="board-column-title">
                        <span class=(status_badge_class(status))>(status)</span>
                        " " <span class="mono">(
                            issues.iter().filter(|issue| issue.status == status).count()
                        )</span>
                    </h2>
                    <ul class="list">
                        for issue in issues.iter().filter(|issue| issue.status == status) {
                            <li class="board-card" data-issue-id=(issue.id.clone())>
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
        <script src="/static/board.js"></script>
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
                        <a href=(with_lang(&format!("/companies/{company_id}/skills/{}", skill.id), lang))>
                            <strong>(skill.name)</strong>
                        </a>
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

/// Goals list page for one company.
#[page("/companies/{company_id}/goals")]
pub async fn goals(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let goal_rows = state
        .goals
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let goal_statuses = ["planned", "active", "achieved", "cancelled"];
    let levels = ["company", "team", "agent", "task"];
    view! {
        <h1 class="page-title">(t(lang, "nav.goals"))</h1>
        <section>
            <h2>(t(lang, "pages.goals.addGoal"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/goals/ui"), lang))>
                <label>(t(lang, "goals.titleLabel"))</label>
                <input type="text" name="title" required="required">
                <label>(t(lang, "goals.descriptionLabel"))</label>
                <input type="text" name="description">
                <label>(t(lang, "goals.levelLabel"))</label>
                <select name="level">
                    for level in levels {
                        <option value=(level)>(level)</option>
                    }
                </select>
                <label>(t(lang, "goals.statusLabel"))</label>
                <select name="status">
                    for status in goal_statuses {
                        <option value=(status)>(status)</option>
                    }
                </select>
                <label>(t(lang, "goals.ownerLabel"))</label>
                <select name="owner_agent_id">
                    <option value="">(t(lang, "common.none"))</option>
                    for agent in agent_rows {
                        <option value=(agent.id.clone())>(agent.name.clone())</option>
                    }
                </select>
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "section.goals"))</h2>
            if goal_rows.is_empty() {
                <p class="empty">(t(lang, "pages.goals.none"))</p>
            } else {
                <ul class="list">
                    for goal in goal_rows {
                        <li>
                            <a href=(with_lang(&format!("/goals/{}", goal.id), lang))>
                                <strong>(goal.title)</strong>
                            </a>
                            " " <span class="badge badge-default">(goal.level)</span>
                            " " <span class=(status_badge_class(&goal.status))>(goal.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Goal detail: attributes, edit form, subgoals, and linked projects.
#[page("/goals/{goal_id}")]
pub async fn goal_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let goal_id = path_param::<GoalId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(goal) = state.goals.get(&goal_id).await.map_err(to_topcoat_error)? else {
        return Err(topcoat::router::error::not_found().into());
    };
    let company_id = goal.company_id.clone();
    let all_goals = state
        .goals
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let children = all_goals
        .iter()
        .filter(|other| other.parent_id.as_deref() == Some(goal_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let project_rows = state
        .projects
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?
        .into_iter()
        .filter(|project| project.goal_id.as_deref() == Some(goal_id.as_str()))
        .collect::<Vec<_>>();
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let goal_statuses = ["planned", "active", "achieved", "cancelled"];
    let levels = ["company", "team", "agent", "task"];
    view! {
        <h1 class="page-title">(goal.title.clone())</h1>
        <p class="mono">(goal.id.clone())</p>
        <p>
            <span class=(status_badge_class(&goal.status))>(goal.status.clone())</span>
            " " <span class="badge badge-default">(goal.level.clone())</span>
            if let Some(description) = &goal.description {
                " " <span class="meta-row">(description.clone())</span>
            }
        </p>
        <section>
            <h2>(t(lang, "goals.edit"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/goals/{goal_id}/edit/ui"), lang))>
                <label>(t(lang, "goals.titleLabel"))</label>
                <input type="text" name="title" value=(goal.title.clone()) required="required">
                <label>(t(lang, "goals.descriptionLabel"))</label>
                <input type="text" name="description" value=(goal.description.clone().unwrap_or_default())>
                <label>(t(lang, "goals.levelLabel"))</label>
                <select name="level">
                    for level in levels {
                        if level == goal.level {
                            <option value=(level) selected="selected">(level)</option>
                        } else {
                            <option value=(level)>(level)</option>
                        }
                    }
                </select>
                <label>(t(lang, "goals.parentLabel"))</label>
                <select name="parent_id">
                    <option value="">(t(lang, "common.none"))</option>
                    for other in &all_goals {
                        if other.id != goal_id {
                            if other.id == goal.parent_id.as_deref().unwrap_or_default() {
                                <option value=(other.id.clone()) selected="selected">(other.title.clone())</option>
                            } else {
                                <option value=(other.id.clone())>(other.title.clone())</option>
                            }
                        }
                    }
                </select>
                <label>(t(lang, "goals.ownerLabel"))</label>
                <select name="owner_agent_id">
                    <option value="">(t(lang, "common.none"))</option>
                    for agent in &agent_rows {
                        if agent.id == goal.owner_agent_id.as_deref().unwrap_or_default() {
                            <option value=(agent.id.clone()) selected="selected">(agent.name.clone())</option>
                        } else {
                            <option value=(agent.id.clone())>(agent.name.clone())</option>
                        }
                    }
                </select>
                <label>(t(lang, "goals.statusLabel"))</label>
                <select name="status">
                    for status in goal_statuses {
                        if status == goal.status {
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
            <h2>(t(lang, "goals.children"))</h2>
            if children.is_empty() {
                <p class="empty">(t(lang, "empty.noGoals"))</p>
            } else {
                <ul class="list">
                    for child in children {
                        <li>
                            <a href=(with_lang(&format!("/goals/{}", child.id), lang))>
                                <strong>(child.title)</strong>
                            </a>
                            " " <span class=(status_badge_class(&child.status))>(child.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "goals.projects"))</h2>
            if project_rows.is_empty() {
                <p class="empty">(t(lang, "empty.noProjects"))</p>
            } else {
                <ul class="list">
                    for project in project_rows {
                        <li>
                            <a href=(with_lang(&format!("/projects/{}", project.id), lang))>
                                <strong>(project.name)</strong>
                            </a>
                            " " <span class=(status_badge_class(&project.status))>(project.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Projects list page for one company.
#[page("/companies/{company_id}/projects")]
pub async fn projects(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let project_rows = state
        .projects
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let goal_rows = state
        .goals
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let project_statuses = [
        "backlog",
        "planned",
        "in_progress",
        "completed",
        "cancelled",
    ];
    view! {
        <h1 class="page-title">(t(lang, "nav.projects"))</h1>
        <section>
            <h2>(t(lang, "pages.projects.addProject"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/projects/ui"), lang))>
                <label>(t(lang, "projects.nameLabel"))</label>
                <input type="text" name="name" required="required">
                <label>(t(lang, "projects.descriptionLabel"))</label>
                <input type="text" name="description">
                <label>(t(lang, "projects.goalLabel"))</label>
                <select name="goal_id">
                    <option value="">(t(lang, "common.none"))</option>
                    for goal in &goal_rows {
                        <option value=(goal.id.clone())>(goal.title.clone())</option>
                    }
                </select>
                <label>(t(lang, "projects.leadLabel"))</label>
                <select name="lead_agent_id">
                    <option value="">(t(lang, "common.none"))</option>
                    for agent in &agent_rows {
                        <option value=(agent.id.clone())>(agent.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "projects.statusLabel"))</label>
                <select name="status">
                    for status in project_statuses {
                        <option value=(status)>(status)</option>
                    }
                </select>
                <label>(t(lang, "projects.targetDateLabel"))</label>
                <input type="text" name="target_date">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "section.projects"))</h2>
            if project_rows.is_empty() {
                <p class="empty">(t(lang, "pages.projects.none"))</p>
            } else {
                <ul class="list">
                    for project in project_rows {
                        <li>
                            <a href=(with_lang(&format!("/projects/{}", project.id), lang))>
                                <strong>(project.name)</strong>
                            </a>
                            " " <span class=(status_badge_class(&project.status))>(project.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Decisions list page for one company.
#[page("/companies/{company_id}/decisions")]
pub async fn decisions(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let decision_rows = state
        .decision_actions
        .list_decisions(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let issue_rows = state
        .issues
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let run_rows = state
        .heartbeat
        .list(&company_id, None, 50)
        .await
        .map_err(to_topcoat_error)?;
    let statuses = ["open", "decided", "cancelled", "expired"];
    view! {
        <h1 class="page-title">(t(lang, "decisions.title"))</h1>
        <section>
            <h2>(t(lang, "decisions.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/decisions/ui"), lang))>
                <label>(t(lang, "decisions.titleLabel"))</label>
                <input type="text" name="title" required="required">
                <label>(t(lang, "decisions.bodyLabel"))</label>
                <input type="text" name="body">
                <label>(t(lang, "decisions.optionsLabel"))</label>
                <input type="text" name="options" placeholder="[{\"id\": \"a\", \"label\": \"A\"}]">
                <label>(t(lang, "decisions.statusLabel"))</label>
                <select name="status">
                    for status in statuses {
                        <option value=(status)>(status)</option>
                    }
                </select>
                <label>(t(lang, "decisions.expiresAtLabel"))</label>
                <input type="text" name="expires_at" placeholder="2026-12-31T00:00:00.000Z">
                <label>(t(lang, "decisions.originAgentLabel"))</label>
                <select name="origin_agent_id">
                    for agent in &agent_rows {
                        <option value=(agent.id.clone())>(agent.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "decisions.originIssueLabel"))</label>
                <select name="origin_issue_id">
                    for issue in &issue_rows {
                        <option value=(issue.id.clone())>(issue.identifier.clone())</option>
                    }
                </select>
                <label>(t(lang, "decisions.originRunLabel"))</label>
                <select name="origin_run_id">
                    for run in &run_rows {
                        <option value=(run.id.clone())>(run.id.clone())</option>
                    }
                </select>
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "section.decisions"))</h2>
            if decision_rows.is_empty() {
                <p class="empty">(t(lang, "decisions.none"))</p>
            } else {
                <ul class="list">
                    for decision in decision_rows {
                        <li>
                            <a href=(with_lang(&format!("/decisions/{}", decision.id), lang))>
                                <strong>(decision.title)</strong>
                            </a>
                            " " <span class=(status_badge_class(&decision.status))>(decision.status)</span>
                            " " <span class="meta-row">(t(lang, "decisions.expires")) ": " (decision.expires_at)</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Decision detail: attributes, resolve form, target issues, effect executions.
#[page("/decisions/{decision_id}")]
pub async fn decision_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let decision_id = path_param::<DecisionId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .decision_actions
        .decision_company(&decision_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let Some(decision) = state
        .decision_actions
        .get_decision(&company_id, &decision_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let target_rows = state
        .decision_actions
        .list_target_issues(&company_id, &decision_id)
        .await
        .map_err(to_topcoat_error)?;
    let effect_rows = state
        .decision_actions
        .list_effect_executions(&company_id, &decision_id)
        .await
        .map_err(to_topcoat_error)?;
    let statuses = ["open", "decided", "cancelled", "expired"];
    view! {
        <h1 class="page-title">(decision.title.clone())</h1>
        <p class="mono">(decision.id.clone())</p>
        <p>
            <span class=(status_badge_class(&decision.status))>(decision.status.clone())</span>
            if let Some(execution_status) = &decision.execution_status {
                " " <span class="badge badge-default">(execution_status.clone())</span>
            }
        </p>
        <p class="meta-row">(decision.body.clone())</p>
        <p class="meta-row">(t(lang, "decisions.options")) ": " (serde_json::to_string(&decision.options).unwrap_or_default())</p>
        <p class="meta-row">(t(lang, "decisions.expires")) ": " (decision.expires_at.clone())</p>
        <section>
            <h2>(t(lang, "decisions.resolve"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/decisions/{decision_id}/resolve/ui"), lang))>
                <label>(t(lang, "decisions.statusLabel"))</label>
                <select name="status">
                    for status in statuses {
                        if status == decision.status {
                            <option value=(status) selected="selected">(status)</option>
                        } else {
                            <option value=(status)>(status)</option>
                        }
                    }
                </select>
                <label>(t(lang, "decisions.chosenOptionLabel"))</label>
                <input type="text" name="chosen_option_id" value=(decision.chosen_option_id.clone().unwrap_or_default())>
                <label>(t(lang, "decisions.decidedByLabel"))</label>
                <input type="text" name="decided_by_user_id" value=(decision.decided_by_user_id.clone().unwrap_or_default())>
                <button type="submit">(t(lang, "settings.save"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "decisions.targetIssues"))</h2>
            if target_rows.is_empty() {
                <p class="empty">(t(lang, "decisions.noTargets"))</p>
            } else {
                <ul class="list">
                    for link in target_rows {
                        <li>
                            <a href=(with_lang(&format!("/issues/{}", link.issue_id), lang))>
                                <span class="mono">(link.issue_id.clone())</span>
                            </a>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "decisions.effectExecutions"))</h2>
            if effect_rows.is_empty() {
                <p class="empty">(t(lang, "decisions.noEffects"))</p>
            } else {
                <ul class="list">
                    for effect in effect_rows {
                        <li>
                            <span class="badge badge-default">(effect.effect_type.clone())</span>
                            " " <span class=(status_badge_class(&effect.status))>(effect.status)</span>
                            " " <span class="meta-row">(t(lang, "decisions.target")) ": " (effect.target_issue_id.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Decision training examples page for one company.
#[page("/companies/{company_id}/decision-training-examples")]
pub async fn training_examples(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let example_rows = state
        .decision_actions
        .list_training_examples(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;
    let issue_rows = state
        .issues
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "training.title"))</h1>
        <section>
            <h2>(t(lang, "training.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/decision-training-examples/ui"), lang))>
                <label>(t(lang, "training.sourceKindLabel"))</label>
                <input type="text" name="source_kind" required="required">
                <label>(t(lang, "training.sourceIdLabel"))</label>
                <input type="text" name="source_id" required="required">
                <label>(t(lang, "training.issueLabel"))</label>
                <select name="issue_id">
                    for issue in &issue_rows {
                        <option value=(issue.id.clone())>(issue.identifier.clone())</option>
                    }
                </select>
                <label>(t(lang, "training.cutoffAtLabel"))</label>
                <input type="text" name="cutoff_at" placeholder="2026-08-01T00:00:00.000Z" required="required">
                <label>(t(lang, "training.snapshotLabel"))</label>
                <input type="text" name="snapshot" placeholder="{}">
                <label>(t(lang, "training.createdByLabel"))</label>
                <input type="text" name="created_by_user_id" value="board">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "training.list"))</h2>
            if example_rows.is_empty() {
                <p class="empty">(t(lang, "training.none"))</p>
            } else {
                <ul class="list">
                    for example in example_rows {
                        <li>
                            <strong>(example.source_kind.clone())</strong>
                            " " <span class="mono">(example.source_id.clone())</span>
                            " " <span class="meta-row">(example.cutoff_at.clone())</span>
                            if let Some(outcome) = &example.decision_outcome {
                                " " <span class="badge badge-default">(outcome.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Status cards page for one company.
#[page("/companies/{company_id}/status-cards")]
pub async fn status_cards(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let card_rows = state
        .scattered
        .list_status_cards(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "statusCards.title"))</h1>
        <section>
            <h2>(t(lang, "statusCards.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/status-cards/ui"), lang))>
                <label>(t(lang, "statusCards.titleLabel"))</label>
                <input type="text" name="title">
                <label>(t(lang, "statusCards.interestPromptLabel"))</label>
                <input type="text" name="interest_prompt" required="required">
                <label>(t(lang, "statusCards.queriesLabel"))</label>
                <input type="text" name="queries" placeholder="[]">
                <label>(t(lang, "statusCards.refreshPolicyLabel"))</label>
                <input type="text" name="refresh_policy" placeholder="{\"interval\": \"15m\"}">
                <label>(t(lang, "statusCards.agentLabel"))</label>
                <select name="agent_id">
                    <option value="">(t(lang, "common.none"))</option>
                    for agent in &agent_rows {
                        <option value=(agent.id.clone())>(agent.name.clone())</option>
                    }
                </select>
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "statusCards.list"))</h2>
            if card_rows.is_empty() {
                <p class="empty">(t(lang, "statusCards.none"))</p>
            } else {
                <ul class="list">
                    for card in card_rows {
                        <li>
                            <strong>(card.title.clone().unwrap_or_else(|| t(lang, "statusCards.untitled")))</strong>
                            " " <span class=(status_badge_class(&card.state))>(card.state)</span>
                            " " <span class="meta-row">(t(lang, "statusCards.changes")) ": " (card.pending_change_count)</span>
                            if card.archived_at.is_none() {
                                " "
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/companies/{company_id}/status-cards/{}/archive/ui", card.id), lang))>
                                    <button type="submit">(t(lang, "statusCards.archive"))</button>
                                </form>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Summary slots page for one company.
#[page("/companies/{company_id}/summary-slots")]
pub async fn summary_slots(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let slot_rows = state
        .scattered
        .list_summary_slots(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "summarySlots.title"))</h1>
        <section>
            <h2>(t(lang, "summarySlots.upsert"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/summary-slots/ui"), lang))>
                <label>(t(lang, "summarySlots.scopeKindLabel"))</label>
                <input type="text" name="scope_kind" required="required">
                <label>(t(lang, "summarySlots.scopeIdLabel"))</label>
                <input type="text" name="scope_id">
                <label>(t(lang, "summarySlots.slotKeyLabel"))</label>
                <input type="text" name="slot_key" required="required">
                <label>(t(lang, "summarySlots.statusLabel"))</label>
                <input type="text" name="status" value="idle">
                <button type="submit">(t(lang, "settings.save"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "summarySlots.list"))</h2>
            if slot_rows.is_empty() {
                <p class="empty">(t(lang, "summarySlots.none"))</p>
            } else {
                <ul class="list">
                    for slot in slot_rows {
                        <li>
                            <span class="badge badge-default">(slot.scope_kind.clone())</span>
                            " " <span class="mono">(slot.slot_key.clone())</span>
                            " " <span class=(status_badge_class(&slot.status))>(slot.status)</span>
                            if let Some(scope_id) = &slot.scope_id {
                                " " <span class="meta-row">(scope_id.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Finance events page for one company.
#[page("/companies/{company_id}/finance-events")]
pub async fn finance_events(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let event_rows = state
        .scattered
        .list_finance_events(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let issue_rows = state
        .issues
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "finance.title"))</h1>
        <section>
            <h2>(t(lang, "finance.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/finance-events/ui"), lang))>
                <label>(t(lang, "finance.eventKindLabel"))</label>
                <input type="text" name="event_kind" required="required">
                <label>(t(lang, "finance.billerLabel"))</label>
                <input type="text" name="biller" required="required">
                <label>(t(lang, "finance.amountLabel"))</label>
                <input type="number" name="amount_cents" required="required">
                <label>(t(lang, "finance.directionLabel"))</label>
                <select name="direction">
                    <option value="debit">"debit"</option>
                    <option value="credit">"credit"</option>
                </select>
                <label>(t(lang, "finance.agentLabel"))</label>
                <select name="agent_id">
                    <option value="">(t(lang, "common.none"))</option>
                    for agent in &agent_rows {
                        <option value=(agent.id.clone())>(agent.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "finance.issueLabel"))</label>
                <select name="issue_id">
                    <option value="">(t(lang, "common.none"))</option>
                    for issue in &issue_rows {
                        <option value=(issue.id.clone())>(issue.identifier.clone())</option>
                    }
                </select>
                <label>(t(lang, "finance.occurredAtLabel"))</label>
                <input type="text" name="occurred_at" placeholder="2026-08-04T00:00:00.000Z" required="required">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "finance.list"))</h2>
            if event_rows.is_empty() {
                <p class="empty">(t(lang, "finance.none"))</p>
            } else {
                <ul class="list">
                    for event in event_rows {
                        <li>
                            <span class="badge badge-default">(event.event_kind.clone())</span>
                            " " <strong>(event.amount_cents)</strong> "¢ "
                            <span class="meta-row">(event.biller.clone())</span>
                            " " <span class="meta-row">(event.occurred_at.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Feedback votes page for one company.
#[page("/companies/{company_id}/feedback-votes")]
pub async fn feedback_votes(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let issue_rows = state
        .issues
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let mut vote_rows = Vec::new();
    for issue in &issue_rows {
        let rows = state
            .scattered
            .list_feedback_votes(&company_id, &issue.id)
            .await
            .map_err(to_topcoat_error)?;
        vote_rows.extend(rows);
    }
    view! {
        <h1 class="page-title">(t(lang, "feedback.title"))</h1>
        <section>
            <h2>(t(lang, "feedback.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/feedback-votes/ui"), lang))>
                <label>(t(lang, "feedback.issueLabel"))</label>
                <select name="issue_id">
                    for issue in &issue_rows {
                        <option value=(issue.id.clone())>(issue.identifier.clone())</option>
                    }
                </select>
                <label>(t(lang, "feedback.targetTypeLabel"))</label>
                <input type="text" name="target_type" required="required">
                <label>(t(lang, "feedback.targetIdLabel"))</label>
                <input type="text" name="target_id" required="required">
                <label>(t(lang, "feedback.authorLabel"))</label>
                <input type="text" name="author_user_id" required="required">
                <label>(t(lang, "feedback.voteLabel"))</label>
                <select name="vote">
                    <option value="up">"up"</option>
                    <option value="down">"down"</option>
                </select>
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "feedback.list"))</h2>
            if vote_rows.is_empty() {
                <p class="empty">(t(lang, "feedback.none"))</p>
            } else {
                <ul class="list">
                    for vote in vote_rows {
                        <li>
                            <span class="badge badge-default">(vote.vote.clone())</span>
                            " " <span class="meta-row">(vote.target_type.clone())</span>
                            " " <span class="mono">(vote.target_id.clone())</span>
                            " " <span class="meta-row">(vote.author_user_id.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Skill detail: versions, policy, comments, stars, and test inputs.
#[page("/companies/{company_id}/skills/{skill_id}")]
pub async fn skill_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<SkillId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(skill) = state
        .skills
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?
        .into_iter()
        .find(|skill| skill.id == skill_id)
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let version_rows = state
        .skill_catalog
        .list_versions(&company_id, &skill_id)
        .await
        .map_err(to_topcoat_error)?;
    let policy = state
        .skill_catalog
        .get_policy(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let comment_rows = state
        .skill_catalog
        .list_comments(&company_id, &skill_id)
        .await
        .map_err(to_topcoat_error)?;
    let star_rows = state
        .skill_catalog
        .list_stars(&company_id, &skill_id)
        .await
        .map_err(to_topcoat_error)?;
    let test_rows = state
        .skill_catalog
        .list_test_inputs(&company_id, &skill_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(skill.name.clone())</h1>
        <p class="meta-row">(skill.description.clone().unwrap_or_default())</p>
        <p class="mono">(skill.id.clone())</p>
        <section>
            <h2>(t(lang, "skillDetail.versions"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/skills/{skill_id}/version/ui"), lang))>
                <input type="text" name="label" placeholder=(t(lang, "skillDetail.versionLabel"))>
                <button type="submit">(t(lang, "skillDetail.publish"))</button>
            </form>
            if version_rows.is_empty() {
                <p class="empty">(t(lang, "skillDetail.noVersions"))</p>
            } else {
                <ul class="list">
                    for version in version_rows {
                        <li>
                            <span class="badge badge-default">(version.revision_number)</span>
                            " " <span class="meta-row">(version.label.clone().unwrap_or_default())</span>
                            " " <span class="meta-row">(version.created_at.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "skillDetail.policy"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/skills/policy/ui"), lang))>
                <input type="text" name="default_effect" value=(policy.as_ref().map(|p| p.default_effect.clone()).unwrap_or_else(|| "allow".to_owned()))>
                <input type="text" name="rules" placeholder="[]">
                <button type="submit">(t(lang, "settings.save"))</button>
            </form>
            if let Some(policy) = &policy {
                <p class="meta-row">(t(lang, "skillDetail.policyRevision")) ": " (policy.revision)</p>
            }
        </section>
        <section>
            <h2>(t(lang, "skillDetail.comments"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/skills/{skill_id}/comments/ui"), lang))>
                <input type="text" name="body" placeholder=(t(lang, "skillDetail.commentPlaceholder"))>
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
            if comment_rows.is_empty() {
                <p class="empty">(t(lang, "skillDetail.noComments"))</p>
            } else {
                <ul class="list">
                    for comment in comment_rows {
                        <li>
                            <span class="meta-row">(comment.body.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "skillDetail.stars"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/skills/{skill_id}/stars/ui"), lang))>
                <input type="text" name="user_id" placeholder=(t(lang, "skillDetail.starUser"))>
                <button type="submit">(t(lang, "skillDetail.star"))</button>
            </form>
            <p class="meta-row">(t(lang, "skillDetail.starCount")) ": " (star_rows.len())</p>
        </section>
        <section>
            <h2>(t(lang, "skillDetail.testInputs"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/skills/{skill_id}/test-inputs/ui"), lang))>
                <input type="text" name="name" placeholder=(t(lang, "skillDetail.testName"))>
                <input type="text" name="content" placeholder=(t(lang, "skillDetail.testContent"))>
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
            if test_rows.is_empty() {
                <p class="empty">(t(lang, "skillDetail.noTestInputs"))</p>
            } else {
                <ul class="list">
                    for test in test_rows {
                        <li>
                            <strong>(test.name.clone())</strong>
                            " " <span class="meta-row">(test.content.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Secret bindings page: provider configs and bindings.
#[page("/companies/{company_id}/secret-bindings")]
pub async fn secret_bindings(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let provider_rows = state
        .secret_bindings
        .list_provider_configs(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let binding_rows = state
        .secret_bindings
        .list_bindings(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "secretBindings.title"))</h1>
        <section>
            <h2>(t(lang, "secretBindings.newProvider"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/secret-bindings/providers/ui"), lang))>
                <label>(t(lang, "secretBindings.providerLabel"))</label>
                <input type="text" name="provider" required="required">
                <label>(t(lang, "secretBindings.displayNameLabel"))</label>
                <input type="text" name="display_name" required="required">
                <label>(t(lang, "secretBindings.configLabel"))</label>
                <input type="text" name="config" placeholder="{}">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
            if provider_rows.is_empty() {
                <p class="empty">(t(lang, "secretBindings.noProviders"))</p>
            } else {
                <ul class="list">
                    for provider in provider_rows {
                        <li>
                            <strong>(provider.display_name.clone())</strong>
                            " " <span class="badge badge-default">(provider.provider.clone())</span>
                            " " <span class=(status_badge_class(&provider.status))>(provider.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "secretBindings.newBinding"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/secret-bindings/bindings/ui"), lang))>
                <label>(t(lang, "secretBindings.secretIdLabel"))</label>
                <input type="text" name="secret_id" required="required">
                <label>(t(lang, "secretBindings.targetLabel"))</label>
                <input type="text" name="target_type" required="required">
                <input type="text" name="target_id" required="required">
                <label>(t(lang, "secretBindings.configPathLabel"))</label>
                <input type="text" name="config_path" required="required">
                <label>(t(lang, "secretBindings.versionSelectorLabel"))</label>
                <input type="text" name="version_selector" value="latest">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
            if binding_rows.is_empty() {
                <p class="empty">(t(lang, "secretBindings.noBindings"))</p>
            } else {
                <ul class="list">
                    for binding in binding_rows {
                        <li>
                            <span class="mono">(binding.secret_id.clone())</span>
                            " " <span class="badge badge-default">(binding.target_type.clone())</span>
                            " " <span class="meta-row">(binding.target_id.clone())</span>
                            " " <span class="meta-row">(binding.config_path.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// User secrets page: definitions and declarations.
#[page("/companies/{company_id}/user-secrets")]
pub async fn user_secrets(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let definition_rows = state
        .secret_bindings
        .list_user_secret_definitions(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let declaration_rows = state
        .secret_bindings
        .list_user_secret_declarations(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "userSecrets.title"))</h1>
        <section>
            <h2>(t(lang, "userSecrets.newDefinition"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/user-secrets/definitions/ui"), lang))>
                <label>(t(lang, "userSecrets.keyLabel"))</label>
                <input type="text" name="key" required="required">
                <label>(t(lang, "userSecrets.nameLabel"))</label>
                <input type="text" name="name" required="required">
                <label>(t(lang, "userSecrets.providerLabel"))</label>
                <input type="text" name="provider" required="required">
                <label>(t(lang, "userSecrets.managedModeLabel"))</label>
                <input type="text" name="managed_mode" value="manual">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
            if definition_rows.is_empty() {
                <p class="empty">(t(lang, "userSecrets.noDefinitions"))</p>
            } else {
                <ul class="list">
                    for definition in definition_rows {
                        <li>
                            <strong>(definition.name.clone())</strong>
                            " " <span class="mono">(definition.key.clone())</span>
                            " " <span class="badge badge-default">(definition.provider.clone())</span>
                            " " <span class=(status_badge_class(&definition.status))>(definition.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "userSecrets.newDeclaration"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/user-secrets/declarations/ui"), lang))>
                <label>(t(lang, "userSecrets.definitionIdLabel"))</label>
                <input type="text" name="user_secret_definition_id" required="required">
                <label>(t(lang, "userSecrets.targetLabel"))</label>
                <input type="text" name="target_type" required="required">
                <input type="text" name="target_id" required="required">
                <label>(t(lang, "userSecrets.envKeyLabel"))</label>
                <input type="text" name="env_key" required="required">
                <label>(t(lang, "userSecrets.configPathLabel"))</label>
                <input type="text" name="config_path" required="required">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
            if declaration_rows.is_empty() {
                <p class="empty">(t(lang, "userSecrets.noDeclarations"))</p>
            } else {
                <ul class="list">
                    for declaration in declaration_rows {
                        <li>
                            <span class="mono">(declaration.user_secret_definition_id.clone())</span>
                            " " <span class="meta-row">(declaration.env_key.clone())</span>
                            " " <span class="meta-row">(declaration.target_type.clone())</span>
                            " " <span class="meta-row">(declaration.target_id.clone())</span>
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

/// Pipelines list page: create form + list.
#[page("/companies/{company_id}/pipelines")]
pub async fn pipelines_list(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let rows = state
        .pipelines
        .list_pipelines(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "pipelines.title"))</h1>
        <form class="inline-form" method="post"
              action=(with_lang(&format!("/companies/{company_id}/pipelines/ui"), lang))>
            <input type="text" name="key" placeholder=(t(lang, "pipelines.key"))>
            <input type="text" name="name" placeholder=(t(lang, "pipelines.name"))>
            <label class="inline-label"><input type="checkbox" name="enforce" value="1"> (t(lang, "pipelines.enforce"))</label>
            <button type="submit">(t(lang, "settings.add"))</button>
        </form>
        if rows.is_empty() {
            <p class="empty">(t(lang, "pipelines.empty"))</p>
        } else {
            <ul class="list">
                for pipeline in rows {
                    <li>
                        <a href=(with_lang(&format!("/pipelines/{}", pipeline.id), lang))>
                            <strong>(pipeline.name)</strong>
                        </a>
                        " " <span class="mono">(pipeline.key)</span>
                        if pipeline.archived_at.is_some() {
                            " " <span class="badge badge-default">"archived"</span>
                        }
                    </li>
                }
            </ul>
        }
    }
}

/// Pipeline detail: stages, transitions, and cases.
#[page("/pipelines/{pipeline_id}")]
pub async fn pipeline_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let pipeline_id = path_param::<PipelinePathId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&pipeline_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let Some(pipeline) = state
        .pipelines
        .get_pipeline(&company_id, &pipeline_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let stages = state
        .pipelines
        .list_stages(&company_id, &pipeline_id)
        .await
        .map_err(to_topcoat_error)?;
    let transitions = state
        .pipelines
        .list_transitions(&company_id, &pipeline_id)
        .await
        .map_err(to_topcoat_error)?;
    let cases = state
        .pipelines
        .list_cases(&company_id, &pipeline_id)
        .await
        .map_err(to_topcoat_error)?;
    let blockers = state
        .pipelines
        .list_blockers(&company_id, &pipeline_id)
        .await
        .map_err(to_topcoat_error)?;
    let pipeline_docs = state
        .pipelines
        .list_pipeline_documents(&company_id, &pipeline_id)
        .await
        .map_err(to_topcoat_error)?;
    let stage_names: std::collections::HashMap<String, String> = stages
        .iter()
        .map(|stage| (stage.id.clone(), stage.name.clone()))
        .collect();
    let stages_url = with_lang(&format!("/pipelines/{pipeline_id}/stages/ui"), lang);
    let transitions_url = with_lang(&format!("/pipelines/{pipeline_id}/transitions/ui"), lang);
    let cases_url = with_lang(&format!("/pipelines/{pipeline_id}/cases/ui"), lang);
    view! {
        <h1 class="page-title">(pipeline.name.clone())</h1>
        <p class="mono">(pipeline.key.clone())</p>
        <section>
            <h2>(t(lang, "pipelines.stages"))</h2>
            <form class="inline-form" method="post" action=(stages_url)>
                <input type="text" name="key" placeholder=(t(lang, "pipelines.key"))>
                <input type="text" name="name" placeholder=(t(lang, "pipelines.name"))>
                <select name="kind">
                    <option value="working">"working"</option>
                    <option value="review">"review"</option>
                    <option value="done">"done"</option>
                    <option value="cancelled">"cancelled"</option>
                </select>
                <input type="number" name="position" value="1" min="0">
                <button type="submit">(t(lang, "settings.add"))</button>
            </form>
            if stages.is_empty() {
                <p class="empty">(t(lang, "pipelines.noStages"))</p>
            } else {
                <ul class="list">
                    for stage in stages.iter() {
                        <li>
                            <span class="badge badge-default">(stage.position)</span>
                            " " <strong>(stage.name.clone())</strong>
                            " " <span class="mono">(stage.key.clone())</span>
                            " " <span class=(status_badge_class(&stage.kind))>(stage.kind.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "pipelines.transitions"))</h2>
            if !stages.is_empty() {
                <form class="inline-form" method="post" action=(transitions_url)>
                    <select name="from_stage_id">
                        for stage in stages.iter() {
                            <option value=(stage.id.clone())>(stage.name.clone())</option>
                        }
                    </select>
                    " → "
                    <select name="to_stage_id">
                        for stage in stages.iter() {
                            <option value=(stage.id.clone())>(stage.name.clone())</option>
                        }
                    </select>
                    <button type="submit">(t(lang, "settings.add"))</button>
                </form>
            }
            if transitions.is_empty() {
                <p class="empty">(t(lang, "pipelines.noTransitions"))</p>
            } else {
                <ul class="list">
                    for transition in transitions {
                        <li>
                            <span class="mono">(
                                stage_names.get(&transition.from_stage_id).cloned().unwrap_or(transition.from_stage_id.clone())
                            )</span>
                            " → "
                            <span class="mono">(
                                stage_names.get(&transition.to_stage_id).cloned().unwrap_or(transition.to_stage_id.clone())
                            )</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "pipelines.cases"))</h2>
            if !stages.is_empty() {
                <form class="inline-form" method="post" action=(cases_url)>
                    <input type="text" name="case_key" placeholder=(t(lang, "pipelines.caseKey"))>
                    <input type="text" name="title" placeholder=(t(lang, "pipelines.caseTitle"))>
                    <select name="stage_id">
                        for stage in stages.iter() {
                            <option value=(stage.id.clone())>(stage.name.clone())</option>
                        }
                    </select>
                    <button type="submit">(t(lang, "settings.add"))</button>
                </form>
            }
            if cases.is_empty() {
                <p class="empty">(t(lang, "pipelines.noCases"))</p>
            } else {
                <ul class="list">
                    for case in cases {
                        <li>
                            <a href=(with_lang(&format!("/pipeline-cases/{}", case.id), lang))>
                                <span class="mono">(case.case_key.clone())</span>
                                " " <strong>(case.title.clone())</strong>
                            </a>
                            " " <span class=(status_badge_class(&case.stage_id))>(
                                stage_names.get(&case.stage_id).cloned().unwrap_or_else(|| "?".to_owned())
                            )</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "pipelines.blockers"))</h2>
            if blockers.is_empty() {
                <p class="empty">(t(lang, "pipelines.noBlockers"))</p>
            } else {
                <ul class="list">
                    for blocker in blockers {
                        <li><span class="mono">(blocker.blocked_by_case_id)</span></li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "pipelines.documents"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/pipelines/{pipeline_id}/documents/ui"), lang))>
                <input type="text" name="document_id" placeholder="document id">
                <input type="text" name="key" placeholder="key">
                <button type="submit">(t(lang, "settings.add"))</button>
            </form>
            if pipeline_docs.is_empty() {
                <p class="empty">(t(lang, "pipelines.noDocuments"))</p>
            } else {
                <ul class="list">
                    for doc in pipeline_docs {
                        <li><span class="mono">(doc.key)</span> " " <span class="meta-row">(doc.document_id)</span></li>
                    }
                </ul>
            }
        </section>
    }
}

/// Pipeline case detail: fields, stage moves, and events.
#[page("/pipeline-cases/{case_id}")]
pub async fn pipeline_case_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let case_id = path_param::<PipelineCasePathId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_case(&case_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let Some(case) = state
        .pipelines
        .get_case(&company_id, &case_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let stages = state
        .pipelines
        .list_stages(&company_id, &case.pipeline_id)
        .await
        .map_err(to_topcoat_error)?;
    let events = state
        .pipelines
        .list_events(&company_id, &case_id)
        .await
        .map_err(to_topcoat_error)?;
    let issue_links = state
        .pipelines
        .list_issue_links(&company_id, &case_id)
        .await
        .map_err(to_topcoat_error)?;
    let blockers = state
        .pipelines
        .list_blockers(&company_id, &case_id)
        .await
        .map_err(to_topcoat_error)?;
    let case_docs = state
        .pipelines
        .list_case_documents(&company_id, &case_id)
        .await
        .map_err(to_topcoat_error)?;
    let automations = state
        .pipelines
        .list_automations(&company_id, &case_id)
        .await
        .map_err(to_topcoat_error)?;
    let stage_names: std::collections::HashMap<String, String> = stages
        .iter()
        .map(|stage| (stage.id.clone(), stage.name.clone()))
        .collect();
    view! {
        <h1 class="page-title">(case.title.clone())</h1>
        <p class="mono">(case.case_key.clone()) " v" (case.version)</p>
        <p>
            <span class=(status_badge_class(&case.stage_id))>(
                stage_names.get(&case.stage_id).cloned().unwrap_or_else(|| "?".to_owned())
            )</span>
            if let Some(kind) = &case.terminal_kind {
                " " <span class="badge badge-default">(kind.clone())</span>
            }
        </p>
        <section>
            <h2>(t(lang, "pipelines.move"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/pipeline-cases/{case_id}/move/ui"), lang))>
                <select name="to_stage_id">
                    for stage in stages.iter() {
                        if stage.id != case.stage_id {
                            <option value=(stage.id.clone())>(stage.name.clone())</option>
                        }
                    }
                </select>
                <button type="submit">(t(lang, "pipelines.move"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "cases.fields"))</h2>
            <p class="mono">(case.fields.to_string())</p>
        </section>
        <section>
            <h2>(t(lang, "pipelines.issueLinks"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/pipeline-cases/{case_id}/issue-links/ui"), lang))>
                <input type="text" name="issue_id" placeholder="issue id">
                <select name="role">
                    <option value="work">"work"</option>
                    <option value="origin">"origin"</option>
                    <option value="conversation">"conversation"</option>
                    <option value="automation">"automation"</option>
                </select>
                <button type="submit">(t(lang, "settings.add"))</button>
            </form>
            if issue_links.is_empty() {
                <p class="empty">(t(lang, "pipelines.noIssueLinks"))</p>
            } else {
                <ul class="list">
                    for link in issue_links {
                        <li><span class="mono">(link.issue_id)</span> " " <span class="badge badge-default">(link.role)</span></li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "pipelines.blockers"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/pipeline-cases/{case_id}/blockers/ui"), lang))>
                <input type="text" name="blocked_by_case_id" placeholder="case id">
                <button type="submit">(t(lang, "settings.add"))</button>
            </form>
            if blockers.is_empty() {
                <p class="empty">(t(lang, "pipelines.noBlockers"))</p>
            } else {
                <ul class="list">
                    for blocker in blockers {
                        <li><span class="mono">(blocker.blocked_by_case_id)</span></li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "pipelines.documents"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/pipeline-cases/{case_id}/documents/ui"), lang))>
                <input type="text" name="document_id" placeholder="document id">
                <input type="text" name="key" placeholder="key">
                <button type="submit">(t(lang, "settings.add"))</button>
            </form>
            if case_docs.is_empty() {
                <p class="empty">(t(lang, "pipelines.noDocuments"))</p>
            } else {
                <ul class="list">
                    for doc in case_docs {
                        <li><span class="mono">(doc.key)</span> " " <span class="meta-row">(doc.document_id)</span></li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "pipelines.automations"))</h2>
            if automations.is_empty() {
                <p class="empty">(t(lang, "pipelines.noAutomations"))</p>
            } else {
                <ul class="list">
                    for automation in automations {
                        <li>
                            <span class="mono">(automation.automation_id)</span>
                            " " <span class=(status_badge_class(&automation.status))>(automation.status)</span>
                            " " <span class="meta-row">(automation.routine_id)</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "pipelines.events"))</h2>
            if events.is_empty() {
                <p class="empty">(t(lang, "pipelines.noEvents"))</p>
            } else {
                <ul class="list">
                    for event in events {
                        <li>
                            <span class="mono">(event.r#type)</span>
                            " " <span class="meta-row">(event.actor_type) " / " (event.created_at)</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// `{pipeline_id}` path parameter for UI pages.
#[path_param(error = bad_request("Invalid pipeline id"))]
pub(crate) struct PipelinePathId(String);

/// `{case_id}` path parameter for pipeline-case UI pages.
#[path_param(error = bad_request("Invalid case id"))]
pub(crate) struct PipelineCasePathId(String);

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
