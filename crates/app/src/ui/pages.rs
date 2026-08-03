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
