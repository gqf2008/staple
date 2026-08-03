//! Board pages: company list, company overview, and issue list.

use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{page, path_param},
    view::view,
};

use crate::state::AppState;

fn to_topcoat_error(error: impl ToString) -> topcoat::Error {
    topcoat::Error::from(std::io::Error::other(error.to_string()))
}

/// Typed `{company_id}` path segment for UI pages.
#[path_param(error = bad_request("Invalid company id"))]
pub(crate) struct CompanyId(String);

/// Home: the company list (company selection context).
#[page("/")]
pub async fn home(cx: &Cx) -> Result {
    let state = app_context::<AppState>(cx);
    let companies = state.companies.list().await.map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">"Companies"</h1>
        if companies.is_empty() {
            <p class="empty">"No companies yet. Create one via the API."</p>
        } else {
            <ul class="list">
                for company in companies {
                    <li>
                        <a href=(format!("/companies/{}", company.id))>
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

        <section>
            <h2>"Goals"</h2>
            if goals.is_empty() {
                <p class="empty">"No goals."</p>
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
            <h2>"Projects"</h2>
            if projects.is_empty() {
                <p class="empty">"No projects."</p>
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
            <h2>"Issues"</h2>
            if issues.is_empty() {
                <p class="empty">"No issues."</p>
            } else {
                <ul class="list">
                    for issue in issues {
                        <li>
                            <span class="mono">(issue.identifier)</span>
                            " " <strong>(issue.title)</strong>
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
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let issues = state
        .issues
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">"Issues"</h1>
        <ul class="list">
            for issue in issues {
                <li>
                    <span class="mono">(issue.identifier)</span>
                    " " <strong>(issue.title)</strong>
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
            " priority: " <span class="mono">(issue.priority)</span>
        </p>
        <p class="meta-row">"company: " (issue.company_id) " | assignee: " (issue.assignee_agent_id.as_deref().unwrap_or("-"))</p>
        if let Some(description) = &issue.description {
            <p>(description)</p>
        }

        <section>
            <h2>"Comments"</h2>
            <form class="inline-form" method="post" action=(format!("/issues/{issue_id}/comments/ui"))>
                <input type="text" name="body" placeholder="Add a comment" required="">
                <button type="submit">"Add"</button>
            </form>
            if comments.is_empty() {
                <p class="empty">"No comments."</p>
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
            <h2>"Documents"</h2>
            if documents.is_empty() {
                <p class="empty">"No documents."</p>
            } else {
                <ul class="list">
                    for document in documents {
                        <li>
                            <strong>(document.title.as_deref().unwrap_or("untitled"))</strong>
                            " rev " <span class="mono">(document.latest_revision_number)</span>
                        </li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>"Attachments"</h2>
            if attachments.is_empty() {
                <p class="empty">"No attachments."</p>
            } else {
                <ul class="list">
                    for attachment in attachments {
                        <li><span class="mono">(attachment.asset_id)</span></li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>"Work products"</h2>
            if work_products.is_empty() {
                <p class="empty">"No work products."</p>
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
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let approvals = state
        .approvals
        .list(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;

    view! {
        <h1 class="page-title">"Approvals"</h1>

        <section>
            <h2>"Request"</h2>
            <form class="inline-form" method="post" action=(format!("/companies/{company_id}/approvals/ui"))>
                <select name="type">
                    <option value="hire_agent">"hire_agent"</option>
                    <option value="approve_ceo_strategy">"approve_ceo_strategy"</option>
                    <option value="budget_override_required">"budget_override_required"</option>
                    <option value="request_board_approval">"request_board_approval"</option>
                </select>
                <input type="text" name="payload" placeholder="{}">
                <button type="submit">"Request"</button>
            </form>
        </section>

        <section>
            <h2>"Pending"</h2>
            if approvals.is_empty() {
                <p class="empty">"No approvals."</p>
            } else {
                <ul class="list">
                    for approval in approvals {
                        <li>
                            <strong>(&approval.r#type)</strong>
                            " " <span class=(status_badge_class(&approval.status))>(&approval.status)</span>
                            " " <span class="mono">(&approval.id)</span>
                            if approval.status == "pending" {
                                <form class="inline-form" method="post" action=(format!("/approvals/{}/decide/ui", approval.id))>
                                    <input type="hidden" name="decision" value="approved">
                                    <button type="submit">"Approve"</button>
                                </form>
                                <form class="inline-form" method="post" action=(format!("/approvals/{}/decide/ui", approval.id))>
                                    <input type="hidden" name="decision" value="rejected">
                                    <button type="submit" class="destructive">"Reject"</button>
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
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let entries = state
        .activity
        .list(&company_id, 200)
        .await
        .map_err(to_topcoat_error)?;

    view! {
        <h1 class="page-title">"Audit log"</h1>
        if entries.is_empty() {
            <p class="empty">"No activity."</p>
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

/// Shared `{id}` path parameter for UI pages.
#[path_param(error = bad_request("Invalid id"))]
pub(crate) struct Id(String);
