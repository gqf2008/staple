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

/// Typed `{workspace_id}` path segment for UI pages.
#[path_param(error = bad_request("Invalid workspace id"))]
pub(crate) struct WorkspaceId(String);

/// Typed `{plugin_id}` path segment for UI pages.
#[path_param(error = bad_request("Invalid plugin id"))]
pub(crate) struct PluginId(String);

/// Typed `{connection_id}` path segment for UI pages.
#[path_param(error = bad_request("Invalid connection id"))]
pub(crate) struct ConnectionId(String);

/// Typed `{claim_token}` path segment for UI pages.
#[path_param(error = bad_request("Invalid claim token"))]
pub(crate) struct ClaimToken(String);

/// Typed `{user_slug}` path segment for UI pages.
#[path_param(error = bad_request("Invalid user slug"))]
pub(crate) struct UserSlug(String);

/// Typed `{catalog_ref}` path segment for UI pages.
#[path_param(error = bad_request("Invalid catalog ref"))]
pub(crate) struct CatalogRef(String);

/// Typed `{token}` path segment for UI pages.
#[path_param(error = bad_request("Invalid token"))]
pub(crate) struct InviteToken(String);

/// Home: the company list (company selection context).
#[page("/")]
pub async fn home(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let state = app_context::<AppState>(cx);
    let companies = state.companies.list().await.map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "page.title.companies"))</h1>
        <form class="inline-form" method="post" action=(with_lang("/companies/ui", lang))>
            <input type="text" name="name" placeholder=(t(lang, "companies.nameLabel")) required="required">
            <input type="text" name="description" placeholder=(t(lang, "companies.descriptionLabel"))>
            <input type="number" name="budgetMonthlyCents" value="0" min="0" placeholder=(t(lang, "companies.budgetLabel"))>
            <input type="number" name="attachmentMaxBytes" value="0" min="1" placeholder=(t(lang, "companies.attachmentMaxLabel"))>
            <button type="submit">(t(lang, "companies.create"))</button>
        </form>
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
            <a href=(with_lang(&format!("/companies/{company_id}/folders"), lang))>(t(lang, "folders.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/board/chat"), lang))>(t(lang, "boardChat.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/export-import"), lang))>(t(lang, "exportImport.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/tools/profiles"), lang))>(t(lang, "toolProfiles.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/tools/connections"), lang))>(t(lang, "toolConnections.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/tools/gateways"), lang))>(t(lang, "toolGateways.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/tools/catalog"), lang))>(t(lang, "toolCatalog.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/tools/invocations"), lang))>(t(lang, "toolInvocations.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/my-issues"), lang))>(t(lang, "myIssues.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/what-needs-me"), lang))>(t(lang, "whatNeedsMe.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/timeline"), lang))>(t(lang, "timeline.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/smoke-runs"), lang))>(t(lang, "smoke.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/feedback-exports"), lang))>(t(lang, "feedbackExports.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/approvals"), lang))>(t(lang, "nav.approvals"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/activity"), lang))>(t(lang, "nav.activity"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/cases"), lang))>(t(lang, "cases.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/artifacts"), lang))>(t(lang, "artifacts.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/pipelines"), lang))>(t(lang, "pipelines.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/access"), lang))>(t(lang, "access.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/costs"), lang))>(t(lang, "costs.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/routines"), lang))>(t(lang, "routines.title"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/goals"), lang))>(t(lang, "nav.goals"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/projects"), lang))>(t(lang, "nav.projects"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/secrets"), lang))>(t(lang, "settings.secrets"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/skills"), lang))>(t(lang, "settings.skills"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/plugins"), lang))>(t(lang, "plugins.title"))</a>
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

fn plugin_status_badge_class(status: &str) -> &'static str {
    match status {
        "enabled" | "installed" => "badge badge-done",
        "disabled" | "uninstalled" => "badge badge-paused",
        "error" => "badge badge-blocked",
        _ => "badge badge-default",
    }
}

/// Resolves a plugin display name from the manifest, falling back to the
/// plugin key when the manifest has no display name/name.
fn plugin_display_name(plugin: &staple_data::PluginRecord) -> String {
    plugin
        .manifest_json
        .get("displayName")
        .or_else(|| plugin.manifest_json.get("name"))
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&plugin.plugin_key)
        .to_owned()
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
    let agent_rows = state
        .agents
        .list(&issue.company_id)
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
            <h2>(t(lang, "issue.claim"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/issues/{issue_id}/claim/ui"), lang))>
                <select name="agent_id">
                    for agent in agent_rows {
                        if Some(agent.id.as_str()) == issue.assignee_agent_id.as_deref() {
                            <option value=(agent.id.clone()) selected="selected">(agent.name.clone())</option>
                        } else {
                            <option value=(agent.id.clone())>(agent.name.clone())</option>
                        }
                    }
                </select>
                <button type="submit">(t(lang, "issue.claim"))</button>
            </form>
        </section>

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
                            <a href=(with_lang(&format!("/approvals/{}", approval.id), lang))>
                                <strong>(&approval.r#type)</strong>
                            </a>
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

/// Approval detail: attributes, decide form, and comments.
#[page("/approvals/{id}")]
pub async fn approval_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let approval_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(approval) = state
        .approvals
        .get(&approval_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let company_id = approval.company_id.clone();
    let comment_rows = state
        .infrastructure
        .list_approval_comments(&company_id, &approval_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(approval.r#type.clone())</h1>
        <nav class="nav-row">
            <a href=(with_lang(&format!("/companies/{company_id}/approvals"), lang))>(t(lang, "approvals.title"))</a>
        </nav>
        <p>
            <span class=(status_badge_class(&approval.status))>(approval.status.clone())</span>
            " " <span class="mono">(approval.id.clone())</span>
        </p>
        <section>
            <h2>(t(lang, "approvalDetail.attributes"))</h2>
            <ul class="list">
                <li><strong>(t(lang, "approvalDetail.type"))</strong> " " <span class="mono">(approval.r#type.clone())</span></li>
                <li><strong>(t(lang, "approvalDetail.company"))</strong> " " <span class="mono">(company_id.clone())</span></li>
                if let Some(agent_id) = &approval.requested_by_agent_id {
                    <li><strong>(t(lang, "approvalDetail.requestedByAgent"))</strong> " " <span class="mono">(agent_id.clone())</span></li>
                }
                if let Some(user_id) = &approval.requested_by_user_id {
                    <li><strong>(t(lang, "approvalDetail.requestedByUser"))</strong> " " <span class="mono">(user_id.clone())</span></li>
                }
                <li><strong>(t(lang, "approvalDetail.status"))</strong> " " <span class=(status_badge_class(&approval.status))>(approval.status.clone())</span></li>
                <li><strong>(t(lang, "approvalDetail.payload"))</strong> " " <span class="mono">(approval.payload.clone())</span></li>
                if let Some(note) = &approval.decision_note {
                    <li><strong>(t(lang, "approvalDetail.decisionNote"))</strong> " " (note.clone())</li>
                }
                if let Some(user_id) = &approval.decided_by_user_id {
                    <li><strong>(t(lang, "approvalDetail.decidedBy"))</strong> " " <span class="mono">(user_id.clone())</span></li>
                }
                if let Some(decided_at) = &approval.decided_at {
                    <li><strong>(t(lang, "approvalDetail.decidedAt"))</strong> " " <span class="mono">(decided_at.clone())</span></li>
                }
                <li><strong>(t(lang, "approvalDetail.created"))</strong> " " <span class="mono">(approval.created_at.clone())</span></li>
            </ul>
        </section>
        if approval.status == "pending" {
            <section>
                <h2>(t(lang, "approvalDetail.decide"))</h2>
                <form class="stack-form" method="post"
                      action=(with_lang(&format!("/approvals/{approval_id}/decide/ui"), lang))>
                    <label>(t(lang, "approvalDetail.noteLabel"))</label>
                    <textarea name="note" rows="3" cols="60" placeholder=(t(lang, "approvalDetail.notePlaceholder"))></textarea>
                    <p class="meta-row">(t(lang, "approvalDetail.decideHint"))</p>
                    <button type="submit" name="decision" value="approved">(t(lang, "approvalDetail.approve"))</button>
                    <button type="submit" name="decision" value="rejected" class="destructive">(t(lang, "approvalDetail.reject"))</button>
                </form>
            </section>
        }
        <section>
            <h2>(t(lang, "approvalDetail.comments"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/approvals/{approval_id}/comments/ui"), lang))>
                <label>(t(lang, "approvalDetail.addComment"))</label>
                <textarea name="body" rows="3" cols="60" required="required"></textarea>
                <button type="submit">(t(lang, "approvalDetail.postComment"))</button>
            </form>
            if comment_rows.is_empty() {
                <p class="empty">(t(lang, "approvalDetail.noComments"))</p>
            } else {
                <ul class="list">
                    for comment in comment_rows {
                        <li>
                            <span class="meta-row">(comment.author_user_id.clone().unwrap_or_else(|| "board".to_owned()))</span>
                            " " <span class="mono">(comment.created_at.clone())</span>
                            <p>(comment.body.clone())</p>
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
        <form class="inline-form" method="post"
              action=(with_lang(&format!("/companies/{company_id}/agents/ui"), lang))>
            <input type="text" name="name" placeholder=(t(lang, "agents.nameLabel")) required="required">
            <select name="role">
                <option value="general">"general"</option>
                <option value="ceo">"ceo"</option>
                <option value="cto">"cto"</option>
                <option value="cmo">"cmo"</option>
                <option value="cfo">"cfo"</option>
                <option value="security">"security"</option>
                <option value="engineer">"engineer"</option>
                <option value="designer">"designer"</option>
                <option value="pm">"pm"</option>
                <option value="qa">"qa"</option>
                <option value="devops">"devops"</option>
                <option value="researcher">"researcher"</option>
            </select>
            <input type="text" name="title" placeholder=(t(lang, "agents.titleLabel"))>
            <input type="text" name="adapter_type" value="cli_local" placeholder=(t(lang, "agents.adapterTypeLabel"))>
            <input type="number" name="budgetMonthlyCents" value="0" min="0" placeholder=(t(lang, "agents.budgetLabel"))>
            <input type="text" name="reports_to" placeholder=(t(lang, "agents.reportsToLabel"))>
            <button type="submit">(t(lang, "agents.create"))</button>
        </form>
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

/// Artifacts page: work products across a company.
#[page("/companies/{company_id}/artifacts")]
pub async fn artifacts(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let rows = state
        .work_products
        .list_for_company(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "artifacts.title"))</h1>
        if rows.is_empty() {
            <p class="empty">(t(lang, "artifacts.empty"))</p>
        } else {
            <ul class="list">
                for artifact in rows {
                    <li>
                        <span class="mono">(artifact.id)</span>
                        " " <strong>(artifact.title)</strong>
                        " " <span class="badge badge-default">(artifact.r#type)</span>
                        " " <span class="meta-row">(t(lang, "artifacts.issue")) ": " (artifact.issue_id)</span>
                        " " <span class="meta-row">(artifact.created_at)</span>
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
    let tool_profile_rows = state
        .tool_catalog
        .list_profiles(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let tool_connection_rows = state
        .tool_connections
        .list_connections(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let tool_catalog_rows = state
        .tool_catalog
        .list_catalog_entries(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;
    let tool_invocation_rows = state
        .tool_gateway
        .list_invocations(&company_id)
        .await
        .map_err(to_topcoat_error)?;
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
            <h2>(t(lang, "agent.tools"))</h2>
            <p class="meta-row">(t(lang, "agent.toolsHint"))</p>
            <ul class="list">
                <li>(t(lang, "agentTools.profiles")) ": " (tool_profile_rows.len())</li>
                <li>(t(lang, "agentTools.connections")) ": " (tool_connection_rows.len())</li>
                <li>(t(lang, "agentTools.catalog")) ": " (tool_catalog_rows.len())</li>
                <li>(t(lang, "agentTools.invocations")) ": " (tool_invocation_rows.len())</li>
            </ul>
            if tool_profile_rows.is_empty() && tool_connection_rows.is_empty() {
                <p class="empty">(t(lang, "agentTools.none"))</p>
            } else {
                <ul class="list">
                    for profile in tool_profile_rows {
                        <li>
                            <span class="badge badge-default">(t(lang, "agentTools.profile"))</span>
                            " " <strong>(profile.name.clone())</strong>
                            " " <span class=(status_badge_class(&profile.status))>(profile.status)</span>
                        </li>
                    }
                    for connection in tool_connection_rows {
                        <li>
                            <span class="badge badge-default">(t(lang, "agentTools.connection"))</span>
                            " " <strong>(connection.name.clone())</strong>
                            " " <span class=(status_badge_class(&connection.status))>(connection.status)</span>
                        </li>
                    }
                </ul>
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
                        <a href=(with_lang(&format!("/routines/{}", routine.id), lang))>
                            <strong>(routine.title)</strong>
                        </a>
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

/// Routine detail: attributes, manual trigger, triggers, run history, and
/// linked routine documents.
#[page("/routines/{id}")]
pub async fn routine_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let routine_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(routine) = state
        .routines
        .get(&routine_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let company_id = routine.company_id.clone();
    let run_rows = state
        .routines
        .list_runs(&company_id, &routine_id)
        .await
        .map_err(to_topcoat_error)?;
    let trigger_rows = state
        .routines
        .list_triggers(&company_id, &routine_id)
        .await
        .map_err(to_topcoat_error)?;
    let document_rows = state
        .infrastructure
        .list_routine_documents(&company_id, &routine_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(routine.title.clone())</h1>
        <nav class="nav-row">
            <a href=(with_lang(&format!("/companies/{company_id}/routines"), lang))>(t(lang, "routines.title"))</a>
        </nav>
        <p>
            <span class=(status_badge_class(&routine.status))>(routine.status.clone())</span>
            " " <span class="badge badge-default">(t(lang, "routineDetail.revision")) " " (routine.latest_revision_number)</span>
            " " <span class="mono">(routine.id.clone())</span>
        </p>
        if let Some(description) = &routine.description {
            <p class="meta-row">(description.clone())</p>
        }
        <section>
            <h2>(t(lang, "routineDetail.attributes"))</h2>
            <ul class="list">
                <li><strong>(t(lang, "routineDetail.company"))</strong> " " <span class="mono">(company_id.clone())</span></li>
                if let Some(project_id) = &routine.project_id {
                    <li><strong>(t(lang, "routineDetail.project"))</strong> " " <span class="mono">(project_id.clone())</span></li>
                }
                if let Some(goal_id) = &routine.goal_id {
                    <li><strong>(t(lang, "routineDetail.goal"))</strong> " " <span class="mono">(goal_id.clone())</span></li>
                }
                if let Some(parent_issue_id) = &routine.parent_issue_id {
                    <li><strong>(t(lang, "routineDetail.parentIssue"))</strong> " " <span class="mono">(parent_issue_id.clone())</span></li>
                }
                if let Some(assignee_agent_id) = &routine.assignee_agent_id {
                    <li><strong>(t(lang, "routineDetail.assignee"))</strong> " " <span class="mono">(assignee_agent_id.clone())</span></li>
                }
                <li><strong>(t(lang, "routineDetail.priority"))</strong> " " (routine.priority.clone())</li>
                <li><strong>(t(lang, "routineDetail.concurrency"))</strong> " " (routine.concurrency_policy.clone())</li>
                <li><strong>(t(lang, "routineDetail.catchUp"))</strong> " " (routine.catch_up_policy.clone())</li>
                <li><strong>(t(lang, "routineDetail.variables"))</strong> " " <span class="mono">(routine.variables.clone())</span></li>
                if let Some(last_triggered_at) = &routine.last_triggered_at {
                    <li><strong>(t(lang, "routineDetail.lastTriggered"))</strong> " " <span class="mono">(last_triggered_at.clone())</span></li>
                }
                <li><strong>(t(lang, "routineDetail.created"))</strong> " " <span class="mono">(routine.created_at.clone())</span></li>
            </ul>
        </section>
        <section>
            <h2>(t(lang, "routineDetail.manualTrigger"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/routines/{routine_id}/trigger/ui"), lang))>
                <button type="submit">(t(lang, "routines.trigger"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "routineDetail.triggers"))</h2>
            if trigger_rows.is_empty() {
                <p class="empty">(t(lang, "routineDetail.noTriggers"))</p>
            } else {
                <ul class="list">
                    for trigger in trigger_rows {
                        <li>
                            <span class="badge badge-default">(trigger["scheduleKind"].as_str().unwrap_or(""))</span>
                            " " <span class="mono">(trigger["scheduleExpr"].as_str().unwrap_or(""))</span>
                            if trigger["enabled"].as_bool().unwrap_or(false) {
                                " " <span class="badge badge-done">"enabled"</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "routineDetail.runs"))</h2>
            if run_rows.is_empty() {
                <p class="empty">(t(lang, "routineDetail.noRuns"))</p>
            } else {
                <ul class="list">
                    for run in run_rows {
                        <li>
                            <span class=(status_badge_class(&run.status))>(run.status.clone())</span>
                            " " <span class="mono">(run.id.clone())</span>
                            " " <span class="meta-row">(t(lang, "routineDetail.created")) " " (run.created_at.clone())</span>
                            if let Some(triggered_by) = &run.triggered_by {
                                " " <span class="meta-row">(t(lang, "routineDetail.triggeredBy")) ": " (triggered_by.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "routineDetail.documents"))</h2>
            if document_rows.is_empty() {
                <p class="empty">(t(lang, "routineDetail.noDocuments"))</p>
            } else {
                <ul class="list">
                    for document in document_rows {
                        <li>
                            <span class="mono">(document.document_id.clone())</span>
                            " " <span class="badge badge-default">(document.key.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
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

/// Company plugins: instance registry list, register form, and per-company
/// enable/disable controls.
#[page("/companies/{company_id}/plugins")]
pub async fn company_plugins(cx: &Cx) -> Result {
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
    let plugin_rows = state.plugins.list().await.map_err(to_topcoat_error)?;
    let mut setting_by_plugin: std::collections::HashMap<
        String,
        staple_data::PluginCompanySettingRecord,
    > = std::collections::HashMap::new();
    for plugin in &plugin_rows {
        if let Ok(setting_rows) = state.plugins.list_company_settings(&plugin.id).await
            && let Some(setting) = setting_rows
                .into_iter()
                .find(|setting| setting.company_id == company_id)
        {
            setting_by_plugin.insert(plugin.id.clone(), setting);
        }
    }
    view! {
        <h1 class="page-title">(company.name) " " (t(lang, "plugins.title"))</h1>
        <p class="meta-row">(t(lang, "plugins.registered"))</p>
        <nav class="nav-row">
            <a href=(with_lang(&format!("/companies/{company_id}"), lang))>(t(lang, "common.back"))</a>
        </nav>
        <section>
            <h2>(t(lang, "plugins.register"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/plugins/register/ui"), lang))>
                <label>(t(lang, "plugins.pluginKey"))</label>
                <input type="text" name="plugin_key" required="required">
                <label>(t(lang, "plugins.packageName"))</label>
                <input type="text" name="package_name" required="required">
                <label>(t(lang, "plugins.version"))</label>
                <input type="text" name="version" required="required">
                <label>(t(lang, "plugins.apiVersion"))</label>
                <input type="number" name="api_version" value="1" min="1">
                <label>(t(lang, "plugins.categories"))</label>
                <input type="text" name="categories" placeholder="tools, integrations">
                <label>(t(lang, "plugins.manifestJson"))</label>
                <textarea name="manifest_json" rows="6" cols="60"></textarea>
                <label>(t(lang, "plugins.installOrder"))</label>
                <input type="number" name="install_order">
                <label>(t(lang, "plugins.packagePath"))</label>
                <input type="text" name="package_path">
                <button type="submit">(t(lang, "plugins.register"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "plugins.registered"))</h2>
            if plugin_rows.is_empty() {
                <p class="empty">(t(lang, "plugins.none"))</p>
            } else {
                <ul class="list">
                    for plugin in &plugin_rows {
                        <li>
                            <a href=(with_lang(&format!("/plugins/{}", plugin.id), lang))>
                                <strong>(plugin_display_name(plugin))</strong>
                            </a>
                            " " <span class="mono">(plugin.plugin_key.clone())</span>
                            " " <span class="badge badge-default">(plugin.version.clone())</span>
                            " " <span class=(plugin_status_badge_class(&plugin.status))>(plugin.status.clone())</span>
                            if let Some(error) = &plugin.last_error {
                                " " <span class="meta-row">(error.clone())</span>
                            }
                            if setting_by_plugin
                                .get(&plugin.id)
                                .map(|setting| setting.enabled)
                                .unwrap_or(true)
                            {
                                <span class="badge badge-done">(t(lang, "plugins.enabled"))</span>
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/companies/{company_id}/plugins/{}/settings/ui", plugin.id), lang))>
                                    <input type="hidden" name="enabled" value="0">
                                    <button type="submit" class="destructive">(t(lang, "plugins.disable"))</button>
                                </form>
                            } else {
                                <span class="badge badge-default">(t(lang, "plugins.disabled"))</span>
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/companies/{company_id}/plugins/{}/settings/ui", plugin.id), lang))>
                                    <input type="hidden" name="enabled" value="1">
                                    <button type="submit">(t(lang, "plugins.enable"))</button>
                                </form>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Query for the plugin detail page state section.
#[topcoat::router::query_params]
struct PluginStateQuery {
    /// Scope kind filter.
    #[serde(rename = "scopeKind")]
    scope_kind: Option<String>,
    /// Scope id filter.
    #[serde(rename = "scopeId")]
    scope_id: Option<String>,
    /// Namespace filter.
    namespace: Option<String>,
}

/// Plugin detail: configuration, company settings, runtime state, entities,
/// jobs/runs, logs, webhook deliveries, database namespaces/migrations, and
/// managed resources for one registered plugin.
#[page("/plugins/{plugin_id}")]
pub async fn plugin_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let plugin_id = path_param::<PluginId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(plugin) = state
        .plugins
        .get(&plugin_id)
        .await
        .map_err(to_topcoat_error)?
    else {
        return Err(topcoat::router::error::not_found().into());
    };
    let config_rows = state
        .plugins
        .list_configs(&plugin_id)
        .await
        .map_err(to_topcoat_error)?;
    let setting_rows = state
        .plugins
        .list_company_settings(&plugin_id)
        .await
        .map_err(to_topcoat_error)?;
    let state_query = topcoat::router::query_params::<PluginStateQuery>(cx).ok();
    let scope_kind = state_query
        .as_ref()
        .and_then(|query| query.scope_kind.clone())
        .unwrap_or_else(|| "instance".to_owned());
    let scope_id = state_query
        .as_ref()
        .and_then(|query| query.scope_id.clone());
    let namespace = state_query
        .as_ref()
        .and_then(|query| query.namespace.clone())
        .unwrap_or_else(|| "default".to_owned());
    let state_rows = state
        .plugin_runtime
        .state_list(&plugin_id, &scope_kind, scope_id.as_deref(), &namespace)
        .await
        .map_err(to_topcoat_error)?;
    let entity_rows = state
        .plugin_runtime
        .entity_list(&plugin_id, None)
        .await
        .map_err(to_topcoat_error)?;
    let job_rows = state
        .plugin_runtime
        .job_list(&plugin_id)
        .await
        .map_err(to_topcoat_error)?;
    let run_rows = state
        .plugin_runtime
        .job_run_list(&plugin_id, None)
        .await
        .map_err(to_topcoat_error)?;
    let log_rows = state
        .plugin_runtime
        .log_list(&plugin_id, None)
        .await
        .map_err(to_topcoat_error)?;
    let webhook_rows = state
        .plugin_runtime
        .webhook_list(&plugin_id, None)
        .await
        .map_err(to_topcoat_error)?;
    let namespace_rows = state
        .plugin_runtime
        .namespace_list(&plugin_id)
        .await
        .map_err(to_topcoat_error)?;
    let migration_rows = state
        .plugin_runtime
        .migration_list(&plugin_id)
        .await
        .map_err(to_topcoat_error)?;
    let company_rows = state.companies.list().await.map_err(to_topcoat_error)?;
    let mut resource_rows: Vec<(String, staple_data::PluginManagedResourceRecord)> = Vec::new();
    for company in &company_rows {
        if let Ok(resources) = state
            .plugins
            .list_managed_resources(&plugin_id, &company.id)
            .await
        {
            for resource in resources {
                resource_rows.push((company.name.clone(), resource));
            }
        }
    }
    let manifest_json = serde_json::to_string_pretty(&plugin.manifest_json).unwrap_or_default();
    view! {
        <h1 class="page-title">(plugin_display_name(&plugin))</h1>
        <p class="meta-row">
            (t(lang, "plugins.pluginKey")) ": " <span class="mono">(plugin.plugin_key.clone())</span>
            " | " (t(lang, "plugins.version")) ": " (plugin.version.clone())
            " | " (t(lang, "plugins.status")) ": "
            <span class=(plugin_status_badge_class(&plugin.status))>(plugin.status.clone())</span>
        </p>
        <p class="mono">(plugin.id.clone())</p>
        <nav class="nav-row">
            <a href=(with_lang("/", lang))>(t(lang, "common.back"))</a>
            <a href=(with_lang("/instance/settings", lang))>(t(lang, "instance.title"))</a>
        </nav>

        <section>
            <h2>(t(lang, "plugins.updateStatus"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang(&format!("/plugins/{plugin_id}/status/ui"), lang))>
                <select name="status">
                    <option value="installed" selected=(plugin.status == "installed")>"installed"</option>
                    <option value="enabled" selected=(plugin.status == "enabled")>"enabled"</option>
                    <option value="disabled" selected=(plugin.status == "disabled")>"disabled"</option>
                    <option value="error" selected=(plugin.status == "error")>"error"</option>
                    <option value="uninstalled" selected=(plugin.status == "uninstalled")>"uninstalled"</option>
                </select>
                <input type="text" name="last_error" value=(plugin.last_error.clone().unwrap_or_default())
                       placeholder=(t(lang, "plugins.lastError"))>
                <button type="submit">(t(lang, "plugins.save"))</button>
            </form>
            if let Some(error) = &plugin.last_error {
                <p class="meta-row">(t(lang, "plugins.lastError")) ": " (error.clone())</p>
            }
            <p class="mono">(manifest_json)</p>
        </section>

        <section>
            <h2>(t(lang, "plugins.config"))</h2>
            if config_rows.is_empty() {
                <p class="empty">(t(lang, "plugins.configNone"))</p>
            } else {
                <ul class="list">
                    for config in &config_rows {
                        <li>
                            <span class="mono">(config.company_id.clone())</span>
                            " " <span class="meta-row">(serde_json::to_string(&config.config_json).unwrap_or_default())</span>
                            " " <span class="meta-row">(config.created_at.clone())</span>
                            if let Some(error) = &config.last_error {
                                " " <span class="meta-row">(error.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/plugins/{plugin_id}/configs/ui"), lang))>
                <label>(t(lang, "plugins.company"))</label>
                <input type="text" name="company_id" required="required">
                <label>(t(lang, "plugins.valueJson"))</label>
                <textarea name="config_json" rows="4" cols="60"></textarea>
                <button type="submit">(t(lang, "plugins.upsertConfig"))</button>
            </form>
        </section>

        <section>
            <h2>(t(lang, "plugins.companySettings"))</h2>
            if setting_rows.is_empty() {
                <p class="empty">(t(lang, "plugins.companySettingsNone"))</p>
            } else {
                <ul class="list">
                    for setting in &setting_rows {
                        <li>
                            <span class="mono">(setting.company_id.clone())</span>
                            " " <span class=(plugin_status_badge_class(if setting.enabled { "enabled" } else { "disabled" }))>
                                (if setting.enabled { "enabled" } else { "disabled" })
                            </span>
                            " " <span class="meta-row">(serde_json::to_string(&setting.settings_json).unwrap_or_default())</span>
                        </li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>(t(lang, "plugins.state"))</h2>
            <form class="inline-form" method="get"
                  action=(with_lang(&format!("/plugins/{plugin_id}"), lang))>
                <input type="text" name="scopeKind" value=(scope_kind.clone()) placeholder=(t(lang, "plugins.scopeKind"))>
                <input type="text" name="scopeId" value=(scope_id.clone().unwrap_or_default()) placeholder=(t(lang, "plugins.scopeId"))>
                <input type="text" name="namespace" value=(namespace.clone()) placeholder=(t(lang, "plugins.namespace"))>
                <button type="submit">(t(lang, "plugins.stateFilter"))</button>
            </form>
            if state_rows.is_empty() {
                <p class="empty">(t(lang, "plugins.stateNone"))</p>
            } else {
                <ul class="list">
                    for state_row in &state_rows {
                        <li>
                            <span class="mono">(state_row.scope_kind.clone())</span>
                            " " <span class="meta-row">(state_row.scope_id.clone().unwrap_or_else(|| "-".to_owned()))</span>
                            " " <span class="meta-row">(state_row.namespace.clone())</span>
                            " " <span class="mono">(state_row.state_key.clone())</span>
                            " " <span class="meta-row">(serde_json::to_string(&state_row.value_json).unwrap_or_default())</span>
                        </li>
                    }
                </ul>
            }
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/plugins/{plugin_id}/state/ui"), lang))>
                <label>(t(lang, "plugins.scopeKind"))</label>
                <input type="text" name="scope_kind" value=(scope_kind.clone())>
                <label>(t(lang, "plugins.scopeId"))</label>
                <input type="text" name="scope_id" value=(scope_id.clone().unwrap_or_default())>
                <label>(t(lang, "plugins.namespace"))</label>
                <input type="text" name="namespace" value=(namespace.clone())>
                <label>(t(lang, "plugins.stateKey"))</label>
                <input type="text" name="key" required="required">
                <label>(t(lang, "plugins.valueJson"))</label>
                <textarea name="value" rows="4" cols="60"></textarea>
                <button type="submit">(t(lang, "plugins.setState"))</button>
            </form>
        </section>

        <section>
            <h2>(t(lang, "plugins.entities"))</h2>
            if entity_rows.is_empty() {
                <p class="empty">(t(lang, "plugins.entitiesNone"))</p>
            } else {
                <ul class="list">
                    for entity in &entity_rows {
                        <li>
                            <span class="mono">(entity.entity_type.clone())</span>
                            " " <strong>(entity.title.clone().unwrap_or_else(|| "-".to_owned()))</strong>
                            " " <span class="meta-row">(entity.scope_kind.clone())</span>
                            " " <span class="meta-row">(entity.scope_id.clone().unwrap_or_else(|| "-".to_owned()))</span>
                            " " <span class="badge badge-default">(entity.status.clone().unwrap_or_else(|| "-".to_owned()))</span>
                            if let Some(external_id) = &entity.external_id {
                                " " <span class="mono">(external_id.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/plugins/{plugin_id}/entities/ui"), lang))>
                <label>(t(lang, "plugins.company"))</label>
                <input type="text" name="company_id">
                <label>(t(lang, "plugins.entityType"))</label>
                <input type="text" name="entity_type" required="required">
                <label>(t(lang, "plugins.scopeKind"))</label>
                <input type="text" name="scope_kind" value="issue">
                <label>(t(lang, "plugins.scopeId"))</label>
                <input type="text" name="scope_id">
                <label>(t(lang, "plugins.externalId"))</label>
                <input type="text" name="external_id">
                <label>(t(lang, "plugins.titleLabel"))</label>
                <input type="text" name="title">
                <label>(t(lang, "plugins.status"))</label>
                <input type="text" name="status">
                <label>(t(lang, "plugins.valueJson"))</label>
                <textarea name="data" rows="4" cols="60"></textarea>
                <button type="submit">(t(lang, "plugins.upsertEntity"))</button>
            </form>
        </section>

        <section>
            <h2>(t(lang, "plugins.jobs"))</h2>
            if job_rows.is_empty() {
                <p class="empty">(t(lang, "plugins.jobsNone"))</p>
            } else {
                <ul class="list">
                    for job in &job_rows {
                        <li>
                            <span class="mono">(job.job_key.clone())</span>
                            " " <span class="meta-row">(job.schedule.clone())</span>
                            " " <span class=(status_badge_class(&job.status))>(job.status.clone())</span>
                            <form class="inline-form" method="post"
                                  action=(with_lang(&format!("/plugins/{plugin_id}/jobs/{}/runs/ui", job.job_key), lang))>
                                <button type="submit">(t(lang, "plugins.runNow"))</button>
                            </form>
                        </li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>(t(lang, "plugins.jobRuns"))</h2>
            if run_rows.is_empty() {
                <p class="empty">(t(lang, "plugins.jobRunsNone"))</p>
            } else {
                <ul class="list">
                    for run in &run_rows {
                        <li>
                            <span class="mono">(run.id.clone())</span>
                            " " <span class="badge badge-default">(run.trigger.clone())</span>
                            " " <span class=(status_badge_class(&run.status))>(run.status.clone())</span>
                            " " <span class="meta-row">(run.started_at.clone().unwrap_or_else(|| "-".to_owned()))</span>
                            if let Some(duration) = run.duration_ms {
                                " " <span class="meta-row">(t(lang, "plugins.durationMs")) ": " (duration)</span>
                            }
                            if let Some(error) = &run.error {
                                " " <span class="meta-row">(error.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>(t(lang, "plugins.logs"))</h2>
            if log_rows.is_empty() {
                <p class="empty">(t(lang, "plugins.logsNone"))</p>
            } else {
                <ul class="list">
                    for log in &log_rows {
                        <li>
                            <span class="badge badge-default">(log.level.clone())</span>
                            " " <span class="meta-row">(log.created_at.clone())</span>
                            " " <strong>(log.message.clone())</strong>
                            if let Some(meta) = &log.meta {
                                " " <span class="meta-row">(serde_json::to_string(meta).unwrap_or_default())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>(t(lang, "plugins.webhooks"))</h2>
            if webhook_rows.is_empty() {
                <p class="empty">(t(lang, "plugins.webhooksNone"))</p>
            } else {
                <ul class="list">
                    for webhook in &webhook_rows {
                        <li>
                            <span class="mono">(webhook.webhook_key.clone())</span>
                            " " <span class=(status_badge_class(&webhook.status))>(webhook.status.clone())</span>
                            " " <span class="meta-row">(webhook.created_at.clone())</span>
                            if let Some(duration) = webhook.duration_ms {
                                " " <span class="meta-row">(t(lang, "plugins.durationMs")) ": " (duration)</span>
                            }
                            if let Some(error) = &webhook.error {
                                " " <span class="meta-row">(error.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>(t(lang, "plugins.database"))</h2>
            if namespace_rows.is_empty() {
                <p class="empty">(t(lang, "plugins.databaseNone"))</p>
            } else {
                <ul class="list">
                    for namespace in &namespace_rows {
                        <li>
                            <span class="mono">(namespace.namespace_name.clone())</span>
                            " " <span class="badge badge-default">(namespace.namespace_mode.clone())</span>
                            " " <span class=(status_badge_class(&namespace.status))>(namespace.status.clone())</span>
                        </li>
                    }
                </ul>
            }
            <h3>(t(lang, "plugins.migrations"))</h3>
            if migration_rows.is_empty() {
                <p class="empty">(t(lang, "plugins.migrationsNone"))</p>
            } else {
                <ul class="list">
                    for migration in &migration_rows {
                        <li>
                            <span class="mono">(migration.namespace_name.clone())</span>
                            " " <span class="mono">(migration.migration_key.clone())</span>
                            " " <span class=(status_badge_class(&migration.status))>(migration.status.clone())</span>
                            " " <span class="meta-row">(migration.checksum.clone())</span>
                            if let Some(error) = &migration.error_message {
                                " " <span class="meta-row">(error.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>

        <section>
            <h2>(t(lang, "plugins.managedResources"))</h2>
            if resource_rows.is_empty() {
                <p class="empty">(t(lang, "plugins.managedResourcesNone"))</p>
            } else {
                <ul class="list">
                    for (company_name, resource) in &resource_rows {
                        <li>
                            <strong>(company_name.clone())</strong>
                            " " <span class="mono">(resource.resource_kind.clone())</span>
                            " " <span class="mono">(resource.resource_key.clone())</span>
                            " " <span class="mono">(resource.resource_id.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
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
    let user_access_rows = state
        .memberships
        .list_users()
        .await
        .map_err(to_topcoat_error)?;
    let company_rows = state.companies.list().await.map_err(to_topcoat_error)?;
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
    let settings_record = state
        .infrastructure
        .get_instance_settings()
        .await
        .map_err(to_topcoat_error)?;
    let general = settings_record.general;
    let general_censor = general
        .get("censorUsernameInLogs")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let general_shortcuts = general
        .get("keyboardShortcuts")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let feedback_preference = general
        .get("feedbackDataSharingPreference")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("prompt")
        .to_owned();
    let backup_retention = general.get("backupRetention");
    let backup_daily_days = backup_retention
        .and_then(|value| value.get("dailyDays"))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(30);
    let backup_weekly_weeks = backup_retention
        .and_then(|value| value.get("weeklyWeeks"))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(12);
    let backup_monthly_months = backup_retention
        .and_then(|value| value.get("monthlyMonths"))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(12);
    let general_json = serde_json::to_string_pretty(&general).unwrap_or_default();
    let experimental = settings_record.experimental;
    let experimental_json = serde_json::to_string_pretty(&experimental).unwrap_or_default();
    view! {
        <h1 class="page-title">(t(lang, "instance.title"))</h1>
        <nav class="nav-row">
            <a href=(with_lang("/profile/settings", lang))>(t(lang, "profile.title"))</a>
            <a href=(with_lang("/users", lang))>(t(lang, "users.title"))</a>
            <a href=(with_lang("/environments", lang))>(t(lang, "environments.title"))</a>
            <a href=(with_lang("/auth", lang))>(t(lang, "auth.title"))</a>
            <a href=(with_lang("/cli-auth", lang))>(t(lang, "cliAuth.title"))</a>
        </nav>
        <section>
            <h2>(t(lang, "instance.general"))</h2>
            <p class="meta-row">(t(lang, "instance.generalHint"))</p>
            <p class="mono">(general_json)</p>
            <form class="stack-form" method="post"
                  action=(with_lang("/instance/settings/general/ui", lang))>
                <label class="inline-label">
                    <input type="checkbox" name="censor_username_in_logs" value="1" checked=(general_censor)>
                    " " (t(lang, "instance.censorUsernameInLogs"))
                </label>
                <label class="inline-label">
                    <input type="checkbox" name="keyboard_shortcuts" value="1" checked=(general_shortcuts)>
                    " " (t(lang, "instance.keyboardShortcuts"))
                </label>
                <label>(t(lang, "instance.feedbackDataSharing"))</label>
                <select name="feedback_data_sharing_preference">
                    <option value="prompt" selected=(feedback_preference == "prompt")>"prompt"</option>
                    <option value="enabled" selected=(feedback_preference == "enabled")>"enabled"</option>
                    <option value="disabled" selected=(feedback_preference == "disabled")>"disabled"</option>
                </select>
                <label>(t(lang, "instance.backupRetention"))</label>
                <input type="number" name="backup_daily_days" value=(backup_daily_days) min="0">
                <input type="number" name="backup_weekly_weeks" value=(backup_weekly_weeks) min="0">
                <input type="number" name="backup_monthly_months" value=(backup_monthly_months) min="0">
                <button type="submit">(t(lang, "settings.save"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "instance.experimental"))</h2>
            <p class="meta-row">(t(lang, "instance.experimentalHint"))</p>
            <p class="mono">(experimental_json.clone())</p>
            <form class="stack-form" method="post"
                  action=(with_lang("/instance/settings/experimental/ui", lang))>
                <label>(t(lang, "instance.experimentalJson"))</label>
                <textarea name="json" rows="6" cols="80">(experimental_json)</textarea>
                <button type="submit">(t(lang, "settings.save"))</button>
            </form>
        </section>
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
            <h2>(t(lang, "instance.userAccess"))</h2>
            if user_access_rows.is_empty() {
                <p class="empty">(t(lang, "users.none"))</p>
            } else {
                <ul class="list">
                    for user_row in &user_access_rows {
                        <li>
                            <span class="mono">(user_row.user_id.clone())</span>
                            " " <span class="meta-row">(user_row.company_count) " companies"</span>
                            if user_row.instance_admin {
                                " " <span class="badge badge-done">(t(lang, "instance.admin"))</span>
                            }
                            <form class="inline-form" method="post"
                                  action=(with_lang(&format!("/instance/users/{}/company-access/ui", user_row.user_id), lang))>
                                for company in &company_rows {
                                    <label class="inline-label">
                                        <input type="checkbox" name="companyIds" value=(company.id.clone())
                                               checked=(user_row.active_company_ids.contains(&company.id))>
                                        " " (company.name.clone())
                                    </label>
                                }
                                <button type="submit">(t(lang, "instance.saveAccess"))</button>
                            </form>
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

/// Profile page query: optional user id to edit.
#[topcoat::router::query_params]
pub(crate) struct ProfileQuery {
    /// User id.
    pub user: Option<String>,
}

/// Profile settings: board user display + sidebar preference.
#[page("/profile/settings")]
pub async fn profile_settings(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let actor_id = crate::auth::current_actor(cx);
    let state = app_context::<AppState>(cx);
    let query = topcoat::router::query_params::<ProfileQuery>(cx)
        .ok()
        .and_then(|params| params.user.clone());
    let user_rows = state
        .infrastructure
        .list_users()
        .await
        .map_err(to_topcoat_error)?;
    let selected_user = query
        .as_deref()
        .and_then(|user_id| user_rows.iter().find(|user| user.id == user_id))
        .or_else(|| user_rows.first());
    let Some(user) = selected_user else {
        return view! {
            <h1 class="page-title">(t(lang, "profile.title"))</h1>
            <nav class="nav-row">
                <a href=(with_lang("/instance/settings", lang))>(t(lang, "instance.title"))</a>
                <a href=(with_lang("/users", lang))>(t(lang, "users.title"))</a>
            </nav>
            <p class="empty">(t(lang, "profile.noUsers"))</p>
        };
    };
    let user_id = user.id.clone();
    let preference = state
        .infrastructure
        .get_user_sidebar_preference(&user_id)
        .await
        .map_err(to_topcoat_error)?;
    let company_order = preference
        .as_ref()
        .and_then(|pref| pref.company_order.as_array())
        .map(|ids| {
            ids.iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    view! {
        <h1 class="page-title">(t(lang, "profile.title"))</h1>
        <nav class="nav-row">
            <a href=(with_lang("/instance/settings", lang))>(t(lang, "instance.title"))</a>
            <a href=(with_lang("/users", lang))>(t(lang, "users.title"))</a>
        </nav>
        <section>
            <h2>(t(lang, "profile.currentOperator"))</h2>
            <p class="meta-row"><strong>(t(lang, "profile.currentOperatorId"))</strong> " " <span class="mono">(actor_id.clone())</span></p>
            <p class="meta-row">(t(lang, "profile.currentOperatorHint"))</p>
            <form class="stack-form" method="get" action=(with_lang("/profile/settings", lang))>
                <label>(t(lang, "profile.switchOperator"))</label>
                <select name="user">
                    <option value="">"board"</option>
                    for user_row in &user_rows {
                        <option value=(user_row.id.clone())>(user_row.name.clone())</option>
                    }
                </select>
                <button type="submit">(t(lang, "profile.switch"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "profile.user"))</h2>
            <ul class="list">
                <li><strong>(t(lang, "profile.id"))</strong> " " <span class="mono">(user.id.clone())</span></li>
                <li><strong>(t(lang, "profile.name"))</strong> " " (user.name.clone())</li>
                <li><strong>(t(lang, "profile.email"))</strong> " " (user.email.clone())</li>
                <li><strong>(t(lang, "profile.emailVerified"))</strong> " " (if user.email_verified { "yes" } else { "no" })</li>
                <li><strong>(t(lang, "profile.created"))</strong> " " <span class="mono">(user.created_at.clone())</span></li>
            </ul>
        </section>
        <section>
            <h2>(t(lang, "profile.sidebarPreference"))</h2>
            <p class="meta-row">(t(lang, "profile.sidebarPreferenceHint"))</p>
            <form class="stack-form" method="post"
                  action=(with_lang("/profile/settings/ui", lang))>
                <input type="hidden" name="user_id" value=(user_id.clone())>
                <label>(t(lang, "profile.companyOrder"))</label>
                <input type="text" name="company_order" value=(company_order) placeholder="company-id-1, company-id-2">
                <button type="submit">(t(lang, "profile.save"))</button>
            </form>
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
                            <a href=(with_lang(&format!("/companies/{company_id}/status-cards/{}/updates", card.id), lang))>
                                <strong>(card.title.clone().unwrap_or_else(|| t(lang, "statusCards.untitled")))</strong>
                            </a>
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

/// Folders page for one company.
#[page("/companies/{company_id}/folders")]
pub async fn folders(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let folder_rows = state
        .infrastructure
        .list_folders(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "folders.title"))</h1>
        <section>
            <h2>(t(lang, "folders.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/folders/ui"), lang))>
                <label>(t(lang, "folders.kindLabel"))</label>
                <input type="text" name="kind" required="required">
                <label>(t(lang, "folders.nameLabel"))</label>
                <input type="text" name="name" required="required">
                <label>(t(lang, "folders.slugLabel"))</label>
                <input type="text" name="slug" required="required">
                <label>(t(lang, "folders.parentLabel"))</label>
                <input type="text" name="parent_id">
                <label>(t(lang, "folders.colorLabel"))</label>
                <input type="text" name="color">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "folders.list"))</h2>
            if folder_rows.is_empty() {
                <p class="empty">(t(lang, "folders.none"))</p>
            } else {
                <ul class="list">
                    for folder in folder_rows {
                        <li>
                            <strong>(folder.name.clone())</strong>
                            " " <span class="badge badge-default">(folder.kind.clone())</span>
                            " " <span class="meta-row">(folder.slug.clone())</span>
                            " "
                            <form class="inline-form" method="post"
                                  action=(with_lang(&format!("/companies/{company_id}/folders/{}/delete/ui", folder.id), lang))>
                                <button type="submit">(t(lang, "common.delete"))</button>
                            </form>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Issue watchdogs page.
#[page("/issues/{id}/watchdogs")]
pub async fn watchdogs(cx: &Cx) -> Result {
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
    let company_id = issue.company_id;
    let watchdog_rows = state
        .infrastructure
        .list_watchdogs(&company_id, &issue_id)
        .await
        .map_err(to_topcoat_error)?;
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "watchdogs.title"))</h1>
        <p class="mono">(issue_id.clone())</p>
        <section>
            <h2>(t(lang, "watchdogs.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/issues/{issue_id}/watchdogs/ui"), lang))>
                <label>(t(lang, "watchdogs.agentLabel"))</label>
                <select name="watchdog_agent_id">
                    for agent in &agent_rows {
                        <option value=(agent.id.clone())>(agent.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "watchdogs.instructionsLabel"))</label>
                <input type="text" name="instructions">
                <label>(t(lang, "watchdogs.statusLabel"))</label>
                <input type="text" name="status" value="active">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "watchdogs.list"))</h2>
            if watchdog_rows.is_empty() {
                <p class="empty">(t(lang, "watchdogs.none"))</p>
            } else {
                <ul class="list">
                    for watchdog in watchdog_rows {
                        <li>
                            <span class="badge badge-default">(watchdog.watchdog_agent_id.clone())</span>
                            " " <span class=(status_badge_class(&watchdog.status))>(watchdog.status)</span>
                            " " <span class="meta-row">(watchdog.instructions.clone().unwrap_or_default())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Users page (instance-level auth users).
#[page("/users")]
pub async fn users(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let state = app_context::<AppState>(cx);
    let user_rows = state
        .infrastructure
        .list_users()
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "users.title"))</h1>
        <section>
            <h2>(t(lang, "users.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang("/users/ui", lang))>
                <label>(t(lang, "users.idLabel"))</label>
                <input type="text" name="id" required="required">
                <label>(t(lang, "users.nameLabel"))</label>
                <input type="text" name="name" required="required">
                <label>(t(lang, "users.emailLabel"))</label>
                <input type="text" name="email" required="required">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "users.list"))</h2>
            if user_rows.is_empty() {
                <p class="empty">(t(lang, "users.none"))</p>
            } else {
                <ul class="list">
                    for user in user_rows {
                        <li>
                            <strong>(user.name.clone())</strong>
                            " " <span class="meta-row">(user.email.clone())</span>
                            " " <span class="mono">(user.id.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Environments page (instance-level).
#[page("/environments")]
pub async fn environments(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let state = app_context::<AppState>(cx);
    let environment_rows = state.environments.list().await.map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "environments.title"))</h1>
        <section>
            <h2>(t(lang, "environments.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang("/environments/ui", lang))>
                <label>(t(lang, "environments.nameLabel"))</label>
                <input type="text" name="name" required="required">
                <label>(t(lang, "environments.descriptionLabel"))</label>
                <input type="text" name="description">
                <label>(t(lang, "environments.driverLabel"))</label>
                <input type="text" name="driver" value="local">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "environments.list"))</h2>
            if environment_rows.is_empty() {
                <p class="empty">(t(lang, "environments.none"))</p>
            } else {
                <ul class="list">
                    for environment in environment_rows {
                        <li>
                            <strong>(environment.name.clone())</strong>
                            " " <span class="badge badge-default">(environment.driver.clone())</span>
                            " " <span class=(status_badge_class(&environment.status))>(environment.status)</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// My issues page: issues actively assigned and in progress.
#[page("/companies/{company_id}/my-issues")]
pub async fn my_issues(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let issue_rows = state
        .issues
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?
        .into_iter()
        .filter(|issue| issue.assignee_agent_id.is_some())
        .collect::<Vec<_>>();
    view! {
        <h1 class="page-title">(t(lang, "myIssues.title"))</h1>
        <section>
            <h2>(t(lang, "myIssues.list"))</h2>
            if issue_rows.is_empty() {
                <p class="empty">(t(lang, "myIssues.none"))</p>
            } else {
                <ul class="list">
                    for issue in issue_rows {
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

/// What needs me page: issues awaiting attention.
#[page("/companies/{company_id}/what-needs-me")]
pub async fn what_needs_me(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let issue_rows = state
        .issues
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?
        .into_iter()
        .filter(|issue| {
            matches!(
                issue.status.as_str(),
                "backlog" | "todo" | "in_review" | "blocked"
            )
        })
        .collect::<Vec<_>>();
    view! {
        <h1 class="page-title">(t(lang, "whatNeedsMe.title"))</h1>
        <section>
            <h2>(t(lang, "whatNeedsMe.list"))</h2>
            if issue_rows.is_empty() {
                <p class="empty">(t(lang, "whatNeedsMe.none"))</p>
            } else {
                <ul class="list">
                    for issue in issue_rows {
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

/// Timeline page: company activity log.
#[page("/companies/{company_id}/timeline")]
pub async fn timeline(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let activity_rows = state
        .activity
        .list(&company_id, 50)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "timeline.title"))</h1>
        <section>
            <h2>(t(lang, "timeline.list"))</h2>
            if activity_rows.is_empty() {
                <p class="empty">(t(lang, "timeline.none"))</p>
            } else {
                <ul class="list">
                    for entry in activity_rows {
                        <li>
                            <span class="mono">(entry.created_at.clone())</span>
                            " " <strong>(entry.action.clone())</strong>
                            " " <span class="meta-row">(entry.entity_type.clone()) "/" (entry.entity_id.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Status card updates page.
#[page("/companies/{company_id}/status-cards/{id}/updates")]
pub async fn status_card_updates(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let card_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let update_rows = state
        .scattered
        .list_status_card_updates(&card_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "statusUpdates.title"))</h1>
        <p class="mono">(card_id.clone())</p>
        <section>
            <h2>(t(lang, "statusUpdates.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/status-cards/{card_id}/updates/ui"), lang))>
                <label>(t(lang, "statusUpdates.kindLabel"))</label>
                <select name="kind">
                    <option value="compile">"compile"</option>
                    <option value="full">"full"</option>
                    <option value="incremental">"incremental"</option>
                </select>
                <label>(t(lang, "statusUpdates.triggerLabel"))</label>
                <select name="trigger">
                    <option value="manual">"manual"</option>
                    <option value="interval">"interval"</option>
                    <option value="reactive">"reactive"</option>
                </select>
                <label>(t(lang, "statusUpdates.statusLabel"))</label>
                <select name="status">
                    <option value="running">"running"</option>
                    <option value="ok">"ok"</option>
                    <option value="failed">"failed"</option>
                </select>
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "statusUpdates.list"))</h2>
            if update_rows.is_empty() {
                <p class="empty">(t(lang, "statusUpdates.none"))</p>
            } else {
                <ul class="list">
                    for update in update_rows {
                        <li>
                            <span class="badge badge-default">(update.kind.clone())</span>
                            " " <span class=(status_badge_class(&update.status))>(update.status)</span>
                            " " <span class="meta-row">(update.started_at.clone())</span>
                            if let Some(summary) = &update.change_summary {
                                " " <span class="meta-row">(summary.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Smoke runs page for one company.
#[page("/companies/{company_id}/smoke-runs")]
pub async fn smoke_runs(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let run_rows = state
        .scattered
        .list_smoke_runs(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "smoke.title"))</h1>
        <section>
            <h2>(t(lang, "smoke.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/smoke-runs/ui"), lang))>
                <label>(t(lang, "smoke.triggerLabel"))</label>
                <select name="trigger">
                    <option value="manual">"manual"</option>
                    <option value="scheduled">"scheduled"</option>
                </select>
                <label>(t(lang, "smoke.statusLabel"))</label>
                <input type="text" name="status" value="running">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "smoke.list"))</h2>
            if run_rows.is_empty() {
                <p class="empty">(t(lang, "smoke.none"))</p>
            } else {
                <ul class="list">
                    for run in run_rows {
                        <li>
                            <span class="badge badge-default">(run.trigger.clone())</span>
                            " " <span class=(status_badge_class(&run.status))>(run.status)</span>
                            " " <span class="meta-row">(run.started_at.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Feedback exports page for one company.
#[page("/companies/{company_id}/feedback-exports")]
pub async fn feedback_exports(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let export_rows = state
        .scattered
        .list_feedback_exports(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let issue_rows = state
        .issues
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "feedbackExports.title"))</h1>
        <section>
            <h2>(t(lang, "feedbackExports.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/feedback-exports/ui"), lang))>
                <label>(t(lang, "feedbackExports.voteIdLabel"))</label>
                <input type="text" name="feedback_vote_id" required="required">
                <label>(t(lang, "feedbackExports.issueLabel"))</label>
                <select name="issue_id">
                    for issue in &issue_rows {
                        <option value=(issue.id.clone())>(issue.identifier.clone())</option>
                    }
                </select>
                <label>(t(lang, "feedbackExports.authorLabel"))</label>
                <input type="text" name="author_user_id" required="required">
                <label>(t(lang, "feedbackExports.targetLabel"))</label>
                <input type="text" name="target_type" required="required">
                <input type="text" name="target_id" required="required">
                <label>(t(lang, "feedbackExports.voteLabel"))</label>
                <select name="vote">
                    <option value="up">"up"</option>
                    <option value="down">"down"</option>
                </select>
                <label>(t(lang, "feedbackExports.targetSummaryLabel"))</label>
                <input type="text" name="target_summary" placeholder="{}">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "feedbackExports.list"))</h2>
            if export_rows.is_empty() {
                <p class="empty">(t(lang, "feedbackExports.none"))</p>
            } else {
                <ul class="list">
                    for export in export_rows {
                        <li>
                            <span class="badge badge-default">(export.target_type.clone())</span>
                            " " <span class="mono">(export.target_id.clone())</span>
                            " " <span class=(status_badge_class(&export.status))>(export.status)</span>
                            " " <span class="meta-row">(export.author_user_id.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Board concierge chat page (streams replies via /api/board/chat/stream).
#[page("/companies/{company_id}/board/chat")]
pub async fn board_chat(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let adapter_names = state.adapters.names();
    view! {
        <h1 class="page-title">(t(lang, "boardChat.title"))</h1>
        <p class="meta-row">(t(lang, "boardChat.hint"))</p>
        <div id="chat-log" class="chat-log"></div>
        <form class="stack-form" id="chat-form">
            <label>(t(lang, "boardChat.adapterLabel"))</label>
            <select name="adapter_type">
                for name in adapter_names {
                    <option value=(name.clone())>(name.clone())</option>
                }
            </select>
            <label>(t(lang, "boardChat.messageLabel"))</label>
            <textarea name="message" rows="4" required="required"></textarea>
            <input type="hidden" name="company_id" value=(company_id)>
            <button type="submit">(t(lang, "boardChat.send"))</button>
        </form>
    }
}

/// Auth / operator page: shows the current actor and lets the operator
/// switch identity (aligned with upstream Auth/UserContext behavior).
#[page("/auth")]
pub async fn auth(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let actor = crate::auth::current_actor(cx);
    let state = app_context::<AppState>(cx);
    let user_rows = state
        .infrastructure
        .list_users()
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "auth.title"))</h1>
        <p class="meta-row">(t(lang, "auth.currentActor")) ": " (actor)</p>
        <section>
            <h2>(t(lang, "auth.switch"))</h2>
            <form class="inline-form" method="get" action=(with_lang("/auth", lang))>
                <select name="user">
                    for user in &user_rows {
                        <option value=(user.id.clone())>(user.name.clone()) " <" (user.email.clone()) ">"</option>
                    }
                </select>
                <button type="submit">(t(lang, "auth.switch"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "auth.users"))</h2>
            if user_rows.is_empty() {
                <p class="empty">(t(lang, "users.none"))</p>
            } else {
                <ul class="list">
                    for user in user_rows {
                        <li>
                            <strong>(user.name.clone())</strong>
                            " " <span class="meta-row">(user.email.clone())</span>
                            " " <span class="mono">(user.id.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// CLI auth page: board API keys and CLI auth challenges.
#[page("/cli-auth")]
pub async fn cli_auth(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let state = app_context::<AppState>(cx);
    let key_rows = state
        .board_keys
        .list_keys()
        .await
        .map_err(to_topcoat_error)?;
    let challenge_rows = state
        .board_keys
        .list_challenges()
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "cliAuth.title"))</h1>
        <section>
            <h2>(t(lang, "cliAuth.newKey"))</h2>
            <form class="inline-form" method="post"
                  action=(with_lang("/cli-auth/keys/ui", lang))>
                <input type="text" name="user_id" placeholder=(t(lang, "cliAuth.userId"))>
                <input type="text" name="name" placeholder=(t(lang, "cliAuth.keyName"))>
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
            <h2>(t(lang, "cliAuth.keys"))</h2>
            if key_rows.is_empty() {
                <p class="empty">(t(lang, "cliAuth.noKeys"))</p>
            } else {
                <ul class="list">
                    for key in key_rows {
                        <li>
                            <strong>(key.name.clone())</strong>
                            " " <span class="mono">(key.id.clone())</span>
                            if key.revoked_at.is_some() {
                                " " <span class="badge badge-paused">"revoked"</span>
                            } else {
                                " " <span class="badge badge-done">"active"</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "cliAuth.newChallenge"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang("/cli-auth/challenges/ui", lang))>
                <label>(t(lang, "cliAuth.commandLabel"))</label>
                <input type="text" name="command" required="required">
                <label>(t(lang, "cliAuth.keyName"))</label>
                <input type="text" name="pending_key_name" required="required">
                <label>(t(lang, "cliAuth.accessLabel"))</label>
                <input type="text" name="requested_access" value="board">
                <label>(t(lang, "cliAuth.companyLabel"))</label>
                <input type="text" name="requested_company_id">
                <button type="submit">(t(lang, "cliAuth.createChallenge"))</button>
            </form>
            <h2>(t(lang, "cliAuth.challenges"))</h2>
            if challenge_rows.is_empty() {
                <p class="empty">(t(lang, "cliAuth.noChallenges"))</p>
            } else {
                <ul class="list">
                    for challenge in challenge_rows {
                        <li>
                            <span class="mono">(challenge.id.clone())</span>
                            if challenge.approved_at.is_some() {
                                " " <span class="badge badge-done">"approved"</span>
                            } else if challenge.cancelled_at.is_some() {
                                " " <span class="badge badge-paused">"cancelled"</span>
                            } else {
                                " " <span class="badge badge-running">"pending"</span>
                            }
                            if challenge.approved_at.is_none() && challenge.cancelled_at.is_none() {
                                " "
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/cli-auth/challenges/{}/approve/ui", challenge.id), lang))>
                                    <button type="submit">(t(lang, "cliAuth.approve"))</button>
                                </form>
                                " "
                                <form class="inline-form" method="post"
                                      action=(with_lang(&format!("/cli-auth/challenges/{}/cancel/ui", challenge.id), lang))>
                                    <button type="submit">(t(lang, "cliAuth.cancel"))</button>
                                </form>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Invite landing page: resolves the plaintext token to an invite and renders
/// company branding, invite metadata, join-request state, and a join entry
/// point (server-rendered mirror of upstream `InviteLanding.tsx`).
#[page("/invite/{invite_token}")]
pub async fn invite_landing(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let token = path_param::<InviteToken>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(invite) = state
        .invites
        .find_by_token(&token)
        .await
        .map_err(to_topcoat_error)?
    else {
        return view! {
            <h1 class="page-title">(t(lang, "inviteLanding.title"))</h1>
            <p class="empty">(t(lang, "inviteLanding.notFound"))</p>
        };
    };
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let revoked = invite.revoked_at.is_some();
    let expired = invite.expires_at.as_str() <= now.as_str();
    let join_request = state
        .invites
        .find_join_request_by_invite(&invite.company_id, &invite.id)
        .await
        .map_err(to_topcoat_error)?;
    if revoked || expired || (invite.accepted_at.is_some() && join_request.is_none()) {
        return view! {
            <h1 class="page-title">(t(lang, "inviteLanding.title"))</h1>
            <p class="empty">(t(lang, "inviteLanding.notFound"))</p>
        };
    }
    let company = state
        .companies
        .get(&invite.company_id)
        .await
        .map_err(to_topcoat_error)?;
    let invite_type = invite.invite_type.clone();
    let allowed_join_types = invite.allowed_join_types.clone();
    let join_type_label = match allowed_join_types.as_str() {
        "human" => t(lang, "inviteLanding.joinTypeHuman"),
        "agent" => t(lang, "inviteLanding.joinTypeAgent"),
        _ => t(lang, "inviteLanding.joinTypeBoth"),
    };
    let human_role = if invite.allowed_join_types == "agent" {
        None
    } else {
        Some(
            invite
                .defaults_payload
                .as_ref()
                .and_then(|payload| payload.get("human"))
                .and_then(|human| human.get("role"))
                .and_then(|value| value.as_str())
                .unwrap_or("operator")
                .to_owned(),
        )
    };
    let invite_message = invite
        .defaults_payload
        .as_ref()
        .and_then(|payload| payload.get("agentMessage"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(str::to_owned);
    let expires_at = invite.expires_at.clone();
    let status = if revoked {
        "revoked"
    } else if invite.accepted_at.is_some() {
        "accepted"
    } else if expired {
        "expired"
    } else {
        "active"
    };
    let status_label = match status {
        "accepted" => t(lang, "inviteLanding.statusAccepted"),
        "expired" => t(lang, "inviteLanding.statusExpired"),
        "revoked" => t(lang, "inviteLanding.statusRevoked"),
        _ => t(lang, "inviteLanding.statusActive"),
    };
    let status_badge = match status {
        "accepted" => "badge badge-done",
        "expired" => "badge badge-paused",
        "revoked" => "badge badge-blocked",
        _ => "badge badge-default",
    };
    let join_status = join_request.as_ref().map(|request| request.status.as_str());
    let join_status_label = match join_status {
        Some("approved") => Some(t(lang, "inviteLanding.requestApproved")),
        Some("rejected") => Some(t(lang, "inviteLanding.requestRejected")),
        Some(_) => Some(t(lang, "inviteLanding.requestPending")),
        None => None,
    };
    let company_id = invite.company_id.clone();
    let company_name = company.as_ref().map(|record| record.name.clone());
    let access_url = with_lang(&format!("/companies/{company_id}/access"), lang);
    view! {
        <h1 class="page-title">(t(lang, "inviteLanding.title"))</h1>
        <p class="meta-row">(t(lang, "inviteLanding.token")) ": " (token)</p>
        if let Some(name) = &company_name {
            <p class="meta-row">(t(lang, "inviteLanding.company")) ": " (name)</p>
        }
        <p class="meta-row">(t(lang, "inviteLanding.inviteType")) ": " (invite_type)</p>
        <p class="meta-row">(t(lang, "inviteLanding.allowedJoinTypes")) ": " (join_type_label)</p>
        if let Some(role) = &human_role {
            <p class="meta-row">(t(lang, "inviteLanding.humanRole")) ": " (role.clone())</p>
        }
        <p class="meta-row">(t(lang, "inviteLanding.expiresAt")) ": " (expires_at)</p>
        <p class="meta-row">
            (t(lang, "inviteLanding.status")) ": " <span class=(status_badge)>(status_label)</span>
        </p>
        if let Some(message) = &invite_message {
            <p class="meta-row">(t(lang, "inviteLanding.message")) ": " (message.clone())</p>
        }
        if let Some(label) = &join_status_label {
            <p class="empty">(label.clone())</p>
        } else if status == "active" && company_name.is_some() {
            <a class="button" href=(access_url)>(t(lang, "inviteLanding.joinCta"))</a>
            <p class="empty">(t(lang, "inviteLanding.joinHint"))</p>
        }
    }
}

/// User profile page (principal based, no auth table).
#[page("/u/{user_slug}")]
pub async fn user_profile(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let user_slug = path_param::<UserSlug>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let profile = state
        .memberships
        .user_profile(&user_slug)
        .await
        .map_err(to_topcoat_error)?;
    let memberships = &profile.memberships;
    view! {
        <h1 class="page-title">(t(lang, "userProfile.title")) ": " (profile.user_id.clone())</h1>
        if profile.instance_admin {
            <p><span class="badge badge-done">(t(lang, "userProfile.admin"))</span></p>
        }
        <section>
            <h2>(t(lang, "userProfile.memberOf"))</h2>
            if memberships.is_empty() {
                <p class="empty">(t(lang, "userProfile.noMemberships"))</p>
            } else {
                <ul class="list">
                    for membership in memberships {
                        <li>
                            <strong>(membership.company_name.clone())</strong>
                            " " <span class=(status_badge_class(&membership.status))>(membership.status.clone())</span>
                            if let Some(role) = &membership.membership_role {
                                " " <span class="badge badge-default">(t(lang, "userProfile.role")) ": " (role.clone())</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "userProfile.created"))</h2>
            <p class="meta-row">(profile.created_issues)</p>
        </section>
        <section>
            <h2>(t(lang, "userProfile.assignedOpen"))</h2>
            <p class="meta-row">(profile.assigned_open_issues)</p>
        </section>
        <section>
            <h2>(t(lang, "userProfile.comments"))</h2>
            <p class="meta-row">(profile.comment_count)</p>
        </section>
    }
}

/// Onboarding page: welcome + first-company creation.
#[page("/onboarding")]
pub async fn onboarding(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    view! {
        <h1 class="page-title">(t(lang, "onboarding.title"))</h1>
        <p class="meta-row">(t(lang, "onboarding.hint"))</p>
        <form class="stack-form" method="post" action=(with_lang("/companies/ui", lang))>
            <label>(t(lang, "onboarding.name"))</label>
            <input type="text" name="name" required="required">
            <label>(t(lang, "onboarding.description"))</label>
            <input type="text" name="description">
            <label>(t(lang, "onboarding.budget"))</label>
            <input type="number" name="budgetMonthlyCents" value="0" min="0">
            <label>(t(lang, "onboarding.attachments"))</label>
            <input type="number" name="attachmentMaxBytes" value="0" min="1">
            <button type="submit">(t(lang, "onboarding.create"))</button>
        </form>
        <a class="button" href=(with_lang("/", lang))>(t(lang, "onboarding.openBoard"))</a>
    }
}

/// Teams catalog browse page.
#[page("/companies/{company_id}/teams/catalog")]
pub async fn team_catalog(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let teams = crate::team_catalog::list();
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let installed_rows = crate::team_catalog::installed(&agent_rows);
    view! {
        <h1 class="page-title">(t(lang, "teamCatalog.title"))</h1>
        if teams.is_empty() {
            <p class="empty">(t(lang, "teamCatalog.empty"))</p>
        } else {
            <ul class="list">
                for team in &teams {
                    <li>
                        <strong>(team.name.clone())</strong>
                        " " <span class="badge badge-default">(team.kind.clone())</span>
                        <p class="meta-row">(team.description.clone())</p>
                        <p class="meta-row">
                            (team.counts.get("agents").and_then(serde_json::Value::as_i64).unwrap_or(0)) " " (t(lang, "teamCatalog.agents"))
                            " · " (team.counts.get("projects").and_then(serde_json::Value::as_i64).unwrap_or(0)) " " (t(lang, "teamCatalog.projects"))
                            " · " (team.counts.get("routines").and_then(serde_json::Value::as_i64).unwrap_or(0)) " " (t(lang, "teamCatalog.routines"))
                            " · " (team.counts.get("localSkills").and_then(serde_json::Value::as_i64).unwrap_or(0)) " " (t(lang, "teamCatalog.skills"))
                        </p>
                        <a class="button" href=(with_lang(&format!("/companies/{company_id}/teams/catalog/{}", team.id), lang))>
                            (t(lang, "teamCatalog.open"))
                        </a>
                    </li>
                }
            </ul>
        }
        <section>
            <h2>(t(lang, "teamCatalog.installed"))</h2>
            if installed_rows.is_empty() {
                <p class="empty">(t(lang, "teamCatalog.noInstalled"))</p>
            } else {
                <ul class="list">
                    for installed_row in &installed_rows {
                        <li>
                            <span class="mono">(installed_row.catalog_id.clone())</span>
                            " " <span class="meta-row">(installed_row.agent_count) " agents"</span>
                            " " <span class=(status_badge_class(if installed_row.out_of_date { "blocked" } else { "done" }))>
                                (if installed_row.out_of_date { t(lang, "teamCatalog.outOfDate") } else { t(lang, "teamCatalog.present") })
                            </span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Team catalog detail page with entrypoint file preview.
#[page("/companies/{company_id}/teams/catalog/{catalog_ref}")]
pub async fn team_catalog_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let catalog_ref = path_param::<CatalogRef>(cx)?.to_string();
    let Some(team) = crate::team_catalog::detail(&catalog_ref) else {
        return Err(topcoat::router::error::not_found().into());
    };
    let file = crate::team_catalog::files(&catalog_ref, "TEAM.md");
    let list_url = with_lang(&format!("/companies/{company_id}/teams/catalog"), lang);
    view! {
        <h1 class="page-title">(t(lang, "teamCatalog.detail")) ": " (team.name.clone())</h1>
        <p class="meta-row">(team.id.clone())</p>
        <p class="meta-row">(team.description.clone())</p>
        <p class="meta-row">
            (team.counts.get("agents").and_then(serde_json::Value::as_i64).unwrap_or(0)) " " (t(lang, "teamCatalog.agents"))
            " · " (team.counts.get("projects").and_then(serde_json::Value::as_i64).unwrap_or(0)) " " (t(lang, "teamCatalog.projects"))
            " · " (team.counts.get("tasks").and_then(serde_json::Value::as_i64).unwrap_or(0)) " " (t(lang, "teamCatalog.tasks"))
            " · " (team.counts.get("routines").and_then(serde_json::Value::as_i64).unwrap_or(0)) " " (t(lang, "teamCatalog.routines"))
        </p>
        <section>
            <h2>(t(lang, "teamCatalog.file")) ": TEAM.md"</h2>
            if let Some(file) = &file {
                <pre class="preview">(file.data.clone())</pre>
            } else {
                <p class="empty">(t(lang, "teamCatalog.empty"))</p>
            }
        </section>
        <form class="stack-form" method="post"
              action=(with_lang(&format!("/companies/{company_id}/teams/catalog/{catalog_ref}/install/ui"), lang))>
            <p class="meta-row">
                (team.agent_slugs.len()) " " (t(lang, "teamCatalog.agents"))
                " · " (team.project_slugs.len()) " " (t(lang, "teamCatalog.projects"))
                " · " (team.required_skills.len()) " " (t(lang, "teamCatalog.skills"))
            </p>
            <button type="submit">(t(lang, "teamCatalog.install"))</button>
        </form>
        <a class="button" href=(list_url)>(t(lang, "teamCatalog.back"))</a>
    }
}

/// Board claim page: inspect the in-memory challenge and claim ownership.
#[page("/board-claim/{claim_token}")]
pub async fn board_claim(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let token = path_param::<ClaimToken>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let code = topcoat::context::try_request_context::<http::request::Parts>(cx)
        .and_then(|parts| {
            parts.uri.query().and_then(|query| {
                query.split('&').find_map(|pair| {
                    let (key, value) = pair.split_once('=')?;
                    (key == "code").then(|| value.to_owned())
                })
            })
        })
        .unwrap_or_default();
    let status = state.board_claim.inspect(&token, Some(&code));
    let claim_url = with_lang(&format!("/board-claim/{token}/claim/ui?code={code}"), lang);
    view! {
        <h1 class="page-title">(t(lang, "boardClaim.title"))</h1>
        if status.status == "available" {
            <p class="meta-row">(t(lang, "boardClaim.available"))</p>
            <form class="stack-form" method="post" action=(claim_url)>
                <button type="submit">(t(lang, "boardClaim.claimButton"))</button>
            </form>
        } else if status.status == "claimed" {
            <p class="empty">(t(lang, "boardClaim.claimed"))</p>
            if let Some(user_id) = &status.claimed_by_user_id {
                <p class="meta-row">(t(lang, "boardClaim.claimedBy")) ": " (user_id.clone())</p>
            }
            <a class="button" href=(with_lang("/", lang))>(t(lang, "boardClaim.openBoard"))</a>
        } else if status.status == "expired" {
            <p class="empty">(t(lang, "boardClaim.expired"))</p>
        } else {
            <p class="empty">(t(lang, "boardClaim.invalid"))</p>
        }
    }
}

/// Company export/import page (JSON manifest baseline).
#[page("/companies/{company_id}/export-import")]
pub async fn export_import(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let result = topcoat::context::try_request_context::<http::request::Parts>(cx)
        .and_then(|parts| {
            parts.uri.query().and_then(|query| {
                query.split('&').find_map(|pair| {
                    let (key, value) = pair.split_once('=')?;
                    (key == "result").then_some(value.to_owned())
                })
            })
        })
        .unwrap_or_default();
    view! {
        <h1 class="page-title">(t(lang, "exportImport.title"))</h1>
        <p class="meta-row">(t(lang, "exportImport.hint"))</p>
        if !result.is_empty() {
            <p class="meta-row">(t(lang, "exportImport.result")) ": " (result)</p>
        }
        <section>
            <h2>(t(lang, "exportImport.export"))</h2>
            <a class="button" href=(with_lang(&format!("/api/companies/{company_id}/export"), lang))>
                (t(lang, "exportImport.download"))
            </a>
        </section>
        <section>
            <h2>(t(lang, "exportImport.import"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/import/ui"), lang))>
                <label>(t(lang, "exportImport.manifestLabel"))</label>
                <textarea name="manifest" rows="12" required="required" placeholder="{}"></textarea>
                <label>(t(lang, "exportImport.strategyLabel"))</label>
                <select name="strategy">
                    <option value="skip">"skip"</option>
                    <option value="overwrite">"overwrite"</option>
                </select>
                <button type="submit">(t(lang, "exportImport.import"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "exportImport.zip"))</h2>
            <p class="meta-row">(t(lang, "exportImport.zipHint"))</p>
            <a class="button" href=(with_lang(&format!("/api/companies/{company_id}/export/archive"), lang))>
                (t(lang, "exportImport.downloadZip"))
            </a>
            <form class="stack-form" id="zip-form">
                <input type="hidden" name="company_id" value=(company_id)>
                <label>(t(lang, "exportImport.zipFile"))</label>
                <input type="file" id="zip-file" name="archive" accept=".zip">
                <label>(t(lang, "exportImport.strategyLabel"))</label>
                <select name="strategy">
                    <option value="skip">"skip"</option>
                    <option value="overwrite">"overwrite"</option>
                </select>
                <button type="submit">(t(lang, "exportImport.preview"))</button>
            </form>
            <div id="zip-preview"></div>
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
                            <a href=(with_lang(&format!("/workspaces/{}", workspace.id), lang))>
                                <strong>(workspace.name)</strong>
                            </a>
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
                            <a href=(with_lang(&format!("/workspaces/{}", workspace.id), lang))>
                                <span class="mono">(workspace.name)</span>
                            </a>
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

/// Workspace detail: project or execution workspace attributes plus
/// execution materialization controls, services, and operations.
#[page("/workspaces/{workspace_id}")]
pub async fn workspace_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let workspace_id = path_param::<WorkspaceId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_rows = state.companies.list().await.map_err(to_topcoat_error)?;
    let mut project_workspace = None;
    let mut execution_workspace = None;
    let mut company_id = String::new();
    'search: for company in &company_rows {
        if let Ok(rows) = state
            .workspaces
            .list_project_workspaces(&company.id, None)
            .await
            && let Some(row) = rows.into_iter().find(|row| row.id == workspace_id)
        {
            project_workspace = Some(row);
            company_id = company.id.clone();
            break 'search;
        }
        if let Ok(rows) = state
            .workspaces
            .list_execution_workspaces(&company.id, None)
            .await
            && let Some(row) = rows.into_iter().find(|row| row.id == workspace_id)
        {
            execution_workspace = Some(row);
            company_id = company.id.clone();
            break 'search;
        }
    }
    let Some(project_workspace) = project_workspace else {
        let Some(execution_workspace) = execution_workspace else {
            return Err(topcoat::router::error::not_found().into());
        };
        let service_rows = state
            .workspaces
            .list_runtime_services(&company_id)
            .await
            .map_err(to_topcoat_error)?;
        let operation_rows = state
            .workspaces
            .list_operations(&company_id)
            .await
            .map_err(to_topcoat_error)?;
        let execution_service_rows: Vec<staple_data::RuntimeServiceRecord> = service_rows
            .into_iter()
            .filter(|service| {
                service.execution_workspace_id.as_deref() == Some(workspace_id.as_str())
            })
            .collect();
        let execution_operation_rows: Vec<staple_data::WorkspaceOperationRecord> = operation_rows
            .into_iter()
            .filter(|operation| {
                operation.execution_workspace_id.as_deref() == Some(workspace_id.as_str())
            })
            .collect();
        return view! {
            <h1 class="page-title">(execution_workspace.name.clone())</h1>
            <nav class="nav-row">
                <a href=(with_lang(&format!("/companies/{company_id}/workspaces"), lang))>(t(lang, "workspaces.title"))</a>
            </nav>
            <p>
                <span class=(status_badge_class(&execution_workspace.status))>(execution_workspace.status.clone())</span>
                " " <span class=(status_badge_class(if execution_workspace.materialized { "done" } else { "backlog" }))>
                    (if execution_workspace.materialized { t(lang, "workspaceDetail.materialized") } else { t(lang, "workspaceDetail.notMaterialized") })
                </span>
            </p>
            <section>
                <h2>(t(lang, "workspaceDetail.attributes"))</h2>
                <ul class="list">
                    <li><strong>(t(lang, "workspaceDetail.id"))</strong> " " <span class="mono">(execution_workspace.id.clone())</span></li>
                    <li><strong>(t(lang, "workspaceDetail.company"))</strong> " " <span class="mono">(company_id.clone())</span></li>
                    <li><strong>(t(lang, "workspaceDetail.project"))</strong> " " <span class="mono">(execution_workspace.project_id.clone())</span></li>
                    if let Some(project_workspace_id) = &execution_workspace.project_workspace_id {
                        <li><strong>(t(lang, "workspaceDetail.projectWorkspace"))</strong> " " <span class="mono">(project_workspace_id.clone())</span></li>
                    }
                    if let Some(source_issue_id) = &execution_workspace.source_issue_id {
                        <li><strong>(t(lang, "workspaceDetail.sourceIssue"))</strong> " " <span class="mono">(source_issue_id.clone())</span></li>
                    }
                    <li><strong>(t(lang, "workspaceDetail.mode"))</strong> " " (execution_workspace.mode.clone())</li>
                    <li><strong>(t(lang, "workspaceDetail.strategy"))</strong> " " (execution_workspace.strategy_type.clone())</li>
                    if let Some(cwd) = &execution_workspace.cwd {
                        <li><strong>(t(lang, "workspaceDetail.cwd"))</strong> " " <span class="mono">(cwd.clone())</span></li>
                    }
                    if let Some(repo_url) = &execution_workspace.repo_url {
                        <li><strong>(t(lang, "workspaceDetail.repoUrl"))</strong> " " <span class="mono">(repo_url.clone())</span></li>
                    }
                    <li><strong>(t(lang, "workspaceDetail.provider"))</strong> " " (execution_workspace.provider_type.clone())</li>
                    if let Some(materialized_at) = &execution_workspace.materialized_at {
                        <li><strong>(t(lang, "workspaceDetail.materializedAt"))</strong> " " <span class="mono">(materialized_at.clone())</span></li>
                    }
                    if let Some(error) = &execution_workspace.materialize_error {
                        <li><strong>(t(lang, "workspaceDetail.materializeError"))</strong> " " <span class="mono">(error.clone())</span></li>
                    }
                    if let Some(secret) = &execution_workspace.credential_secret_name {
                        <li><strong>(t(lang, "workspaceDetail.credentialSecret"))</strong> " " <span class="mono">(secret.clone())</span></li>
                    }
                    <li><strong>(t(lang, "workspaceDetail.created"))</strong> " " <span class="mono">(execution_workspace.created_at.clone())</span></li>
                </ul>
            </section>
            if !execution_workspace.materialized {
                <section>
                    <h2>(t(lang, "workspaceDetail.materialize"))</h2>
                    <form class="inline-form" method="post"
                          action=(with_lang(&format!("/companies/{company_id}/workspaces/{workspace_id}/materialize/ui"), lang))>
                        <button type="submit" class="secondary">(t(lang, "workspaces.materialize"))</button>
                    </form>
                </section>
            }
            <section>
                <h2>(t(lang, "workspaceDetail.services"))</h2>
                if execution_service_rows.is_empty() {
                    <p class="empty">(t(lang, "workspaceDetail.noServices"))</p>
                } else {
                    <ul class="list">
                        for service in execution_service_rows {
                            <li>
                                <strong>(service.service_name.clone())</strong>
                                " " <span class="badge badge-default">(service.status.clone())</span>
                                " " <span class="mono">(service.scope_type.clone())</span>
                                if let Some(url) = &service.url {
                                    " " <span class="mono">(url.clone())</span>
                                }
                            </li>
                        }
                    </ul>
                }
            </section>
            <section>
                <h2>(t(lang, "workspaceDetail.operations"))</h2>
                if execution_operation_rows.is_empty() {
                    <p class="empty">(t(lang, "workspaceDetail.noOperations"))</p>
                } else {
                    <ul class="list">
                        for operation in execution_operation_rows {
                            <li>
                                <span class=(status_badge_class(&operation.phase))>(operation.phase.clone())</span>
                                " " <span class="mono">(operation.command.clone().unwrap_or_default())</span>
                                " " <span class="meta-row">(operation.created_at.clone())</span>
                            </li>
                        }
                    </ul>
                }
            </section>
        };
    };
    view! {
        <h1 class="page-title">(project_workspace.name.clone())</h1>
        <nav class="nav-row">
            <a href=(with_lang(&format!("/companies/{company_id}/workspaces"), lang))>(t(lang, "workspaces.title"))</a>
        </nav>
        <p>
            <span class="badge badge-default">(project_workspace.source_type.clone())</span>
            " " <span class="mono">(project_workspace.id.clone())</span>
        </p>
        <section>
            <h2>(t(lang, "workspaceDetail.attributes"))</h2>
            <ul class="list">
                <li><strong>(t(lang, "workspaceDetail.id"))</strong> " " <span class="mono">(project_workspace.id.clone())</span></li>
                <li><strong>(t(lang, "workspaceDetail.company"))</strong> " " <span class="mono">(company_id.clone())</span></li>
                <li><strong>(t(lang, "workspaceDetail.project"))</strong> " " <span class="mono">(project_workspace.project_id.clone())</span></li>
                <li><strong>(t(lang, "workspaceDetail.sourceType"))</strong> " " (project_workspace.source_type.clone())</li>
                if let Some(cwd) = &project_workspace.cwd {
                    <li><strong>(t(lang, "workspaceDetail.cwd"))</strong> " " <span class="mono">(cwd.clone())</span></li>
                }
                if let Some(repo_url) = &project_workspace.repo_url {
                    <li><strong>(t(lang, "workspaceDetail.repoUrl"))</strong> " " <span class="mono">(repo_url.clone())</span></li>
                }
                if let Some(repo_ref) = &project_workspace.repo_ref {
                    <li><strong>(t(lang, "workspaceDetail.repoRef"))</strong> " " <span class="mono">(repo_ref.clone())</span></li>
                }
                if let Some(default_ref) = &project_workspace.default_ref {
                    <li><strong>(t(lang, "workspaceDetail.defaultRef"))</strong> " " <span class="mono">(default_ref.clone())</span></li>
                }
                <li><strong>(t(lang, "workspaceDetail.visibility"))</strong> " " (project_workspace.visibility.clone())</li>
                <li><strong>(t(lang, "workspaceDetail.primary"))</strong> " " (if project_workspace.is_primary { "yes" } else { "no" })</li>
                <li><strong>(t(lang, "workspaceDetail.created"))</strong> " " <span class="mono">(project_workspace.created_at.clone())</span></li>
            </ul>
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

/// Review queue page: pipeline attention feed (suggestions + reviews) with
/// inline approve/request-changes/reject and accept/dismiss decisions.
#[page("/companies/{company_id}/review-queue")]
pub async fn review_queue(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let attention = state
        .pipelines
        .list_attention(&company_id, 100)
        .await
        .map_err(to_topcoat_error)?;
    let suggestions = attention["suggestions"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let reviews = attention["reviews"].as_array().cloned().unwrap_or_default();
    let decide_url = with_lang(
        &format!("/companies/{company_id}/review-queue/decide/ui"),
        lang,
    );
    let pipelines_url = with_lang(&format!("/companies/{company_id}/pipelines"), lang);
    view! {
        <h1 class="page-title">(t(lang, "reviewQueue.title"))</h1>
        <nav class="nav-row">
            <a href=(pipelines_url)>(t(lang, "pipelines.title"))</a>
        </nav>
        <section>
            <h2>(t(lang, "reviewQueue.suggestions"))</h2>
            if suggestions.is_empty() {
                <p class="empty">(t(lang, "reviewQueue.empty"))</p>
            } else {
                <ul class="list">
                    for suggestion in &suggestions {
                        <li>
                            <a href=(with_lang(&format!("/pipelines/{}/items/{}",
                                suggestion["case"]["pipeline"]["id"].as_str().unwrap_or_default(),
                                suggestion["case"]["id"].as_str().unwrap_or_default()), lang))>
                                <strong>(suggestion["case"]["title"].as_str().unwrap_or_default())</strong>
                            </a>
                            " " <span class="mono">(suggestion["case"]["caseKey"].as_str().unwrap_or_default())</span>
                            " " <span class="badge badge-default">(suggestion["case"]["stage"]["name"].as_str().unwrap_or_default())</span>
                            " → " <span class="badge badge-running">(suggestion["suggestion"]["toStageName"].as_str().unwrap_or_default())</span>
                            <p class="meta-row">(suggestion["suggestion"]["rationale"].as_str().unwrap_or_default())</p>
                            <form class="inline-form" method="post" action=(decide_url.clone())>
                                <input type="hidden" name="caseId" value=(suggestion["case"]["id"].as_str().unwrap_or_default())>
                                <input type="hidden" name="suggestionId" value=(suggestion["suggestion"]["id"].as_str().unwrap_or_default())>
                                <input type="hidden" name="decision" value="accept">
                                <input type="hidden" name="expectedVersion" value=(suggestion["case"]["version"].as_i64().unwrap_or(0))>
                                <button type="submit">(t(lang, "reviewQueue.accept"))</button>
                            </form>
                            <form class="inline-form" method="post" action=(decide_url.clone())>
                                <input type="hidden" name="caseId" value=(suggestion["case"]["id"].as_str().unwrap_or_default())>
                                <input type="hidden" name="suggestionId" value=(suggestion["suggestion"]["id"].as_str().unwrap_or_default())>
                                <input type="hidden" name="decision" value="dismiss">
                                <button type="submit" class="destructive">(t(lang, "reviewQueue.dismiss"))</button>
                            </form>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "reviewQueue.reviews"))</h2>
            if reviews.is_empty() {
                <p class="empty">(t(lang, "reviewQueue.empty"))</p>
            } else {
                <ul class="list">
                    for review in &reviews {
                        <li>
                            <a href=(with_lang(&format!("/pipelines/{}/items/{}",
                                review["case"]["pipeline"]["id"].as_str().unwrap_or_default(),
                                review["case"]["id"].as_str().unwrap_or_default()), lang))>
                                <strong>(review["case"]["title"].as_str().unwrap_or_default())</strong>
                            </a>
                            " " <span class="mono">(review["case"]["caseKey"].as_str().unwrap_or_default())</span>
                            " " <span class="badge badge-default">(review["case"]["stage"]["name"].as_str().unwrap_or_default())</span>
                            <form class="inline-form" method="post" action=(decide_url.clone())>
                                <input type="hidden" name="caseId" value=(review["case"]["id"].as_str().unwrap_or_default())>
                                <input type="hidden" name="decision" value="approve">
                                <input type="hidden" name="expectedVersion" value=(review["review"]["expectedVersion"].as_i64().unwrap_or(0))>
                                <button type="submit">(t(lang, "reviewQueue.approve"))</button>
                            </form>
                            <form class="inline-form" method="post" action=(decide_url.clone())>
                                <input type="hidden" name="caseId" value=(review["case"]["id"].as_str().unwrap_or_default())>
                                <input type="hidden" name="decision" value="request_changes">
                                <input type="hidden" name="expectedVersion" value=(review["review"]["expectedVersion"].as_i64().unwrap_or(0))>
                                <input type="text" name="reason" placeholder=(t(lang, "reviewQueue.reasonLabel"))>
                                <button type="submit">(t(lang, "reviewQueue.requestChanges"))</button>
                            </form>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Learnings page: company-wide learning events grouped by day.
#[page("/companies/{company_id}/learnings")]
pub async fn learnings(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let learning_types = [
        "transition_suggested".to_owned(),
        "suggestion_resolved".to_owned(),
        "review_decided".to_owned(),
        "transition_forced".to_owned(),
        "upstream_drift".to_owned(),
        "drift_acknowledged".to_owned(),
    ];
    let offset = topcoat::context::try_request_context::<http::request::Parts>(cx)
        .and_then(|parts| {
            parts.uri.query().and_then(|query| {
                query.split('&').find_map(|pair| {
                    let (key, value) = pair.split_once('=')?;
                    (key == "offset").then_some(value.to_owned())
                })
            })
        })
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value >= 0)
        .unwrap_or(0);
    let events = state
        .pipelines
        .list_company_case_events(&company_id, &learning_types, 50, offset)
        .await
        .map_err(to_topcoat_error)?;
    let items = events["items"].as_array().cloned().unwrap_or_default();
    let has_more = events["pagination"]["hasMore"].as_bool().unwrap_or(false);
    let mut groups: Vec<(String, Vec<serde_json::Value>)> = Vec::new();
    for item in &items {
        let day = item["createdAt"]
            .as_str()
            .and_then(|value| value.get(..10))
            .unwrap_or("")
            .to_owned();
        match groups.last_mut() {
            Some((last_day, rows)) if *last_day == day => rows.push(item.clone()),
            _ => groups.push((day, vec![item.clone()])),
        }
    }
    let next_offset = offset + items.len() as i64;
    let prev_offset = (offset - 50).max(0);
    let page = offset / 50 + 1;
    view! {
        <h1 class="page-title">(t(lang, "learnings.title"))</h1>
        if groups.is_empty() {
            <p class="empty">(t(lang, "learnings.empty"))</p>
        } else {
            for (day, rows) in &groups {
                <section>
                    <h2 class="mono">(day.clone())</h2>
                    <ul class="list">
                        for row in rows {
                            <li>
                                <a href=(with_lang(&format!("/pipelines/{}/items/{}",
                                    row["pipeline"]["id"].as_str().unwrap_or_default(),
                                    row["case"]["id"].as_str().unwrap_or_default()), lang))>
                                    <strong>(row["case"]["title"].as_str().unwrap_or_default())</strong>
                                </a>
                                " " <span class="mono">(row["type"].as_str().unwrap_or_default())</span>
                                " " <span class="meta-row">(row["case"]["caseKey"].as_str().unwrap_or_default())</span>
                            </li>
                        }
                    </ul>
                </section>
            }
            <nav class="nav-row">
                if offset > 0 {
                    <a href=(with_lang(&format!("/companies/{company_id}/learnings?offset={prev_offset}"), lang))>(t(lang, "learnings.prev"))</a>
                }
                <span class="meta-row">(t(lang, "learnings.page")) " " (page)</span>
                if has_more {
                    <a href=(with_lang(&format!("/companies/{company_id}/learnings?offset={next_offset}"), lang))>(t(lang, "learnings.next"))</a>
                }
            </nav>
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
    let pipeline_id = path_param::<PipelineId>(cx)?.to_string();
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
    let settings_url = with_lang(&format!("/pipelines/{pipeline_id}/settings/ui"), lang);
    let description = pipeline.description.clone().unwrap_or_default();
    view! {
        <h1 class="page-title">(pipeline.name.clone())</h1>
        <p class="mono">(pipeline.key.clone())</p>
        <section>
            <h2>(t(lang, "pipelines.settings"))</h2>
            <form class="inline-form" method="post" action=(settings_url)>
                <input type="text" name="name" value=(pipeline.name.clone()) required="required">
                <input type="text" name="description" value=(description)>
                <select name="status">
                    <option value="active" selected=(pipeline.archived_at.is_none())>"active"</option>
                    <option value="archived" selected=(pipeline.archived_at.is_some())>"archived"</option>
                </select>
                <button type="submit">(t(lang, "settings.save"))</button>
            </form>
        </section>
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

/// Tool profiles page: access profiles and their entries for one company.
#[page("/companies/{company_id}/tools/profiles")]
pub async fn tool_profiles(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let profile_rows = state
        .tool_catalog
        .list_profiles(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let entry_rows = state
        .tool_catalog
        .list_profile_entries(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "toolProfiles.title"))</h1>
        <section>
            <h2>(t(lang, "toolProfiles.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/tools/profiles/ui"), lang))>
                <label>(t(lang, "toolProfiles.keyLabel"))</label>
                <input type="text" name="profile_key" required="required">
                <label>(t(lang, "toolProfiles.nameLabel"))</label>
                <input type="text" name="name" required="required">
                <label>(t(lang, "toolProfiles.descriptionLabel"))</label>
                <input type="text" name="description">
                <label>(t(lang, "toolProfiles.statusLabel"))</label>
                <input type="text" name="status" value="active">
                <label>(t(lang, "toolProfiles.defaultActionLabel"))</label>
                <select name="default_action">
                    <option value="deny">"deny"</option>
                    <option value="allow">"allow"</option>
                </select>
                <label>(t(lang, "toolProfiles.metadataLabel"))</label>
                <input type="text" name="metadata" placeholder="{}">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "toolProfiles.list"))</h2>
            if profile_rows.is_empty() {
                <p class="empty">(t(lang, "toolProfiles.none"))</p>
            } else {
                <ul class="list">
                    for profile in &profile_rows {
                        <li>
                            <strong>(profile.name.clone())</strong>
                            " " <span class="mono">(profile.profile_key.clone())</span>
                            " " <span class=(status_badge_class(&profile.status))>(profile.status.clone())</span>
                            " " <span class="badge badge-default">(profile.default_action.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "toolProfiles.entries"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/tools/profile-entries/ui"), lang))>
                <label>(t(lang, "toolProfiles.profileLabel"))</label>
                <select name="profile_id">
                    for profile in &profile_rows {
                        <option value=(profile.id.clone())>(profile.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "toolProfiles.selectorTypeLabel"))</label>
                <input type="text" name="selector_type" required="required">
                <label>(t(lang, "toolProfiles.effectLabel"))</label>
                <select name="effect">
                    <option value="include">"include"</option>
                    <option value="exclude">"exclude"</option>
                </select>
                <label>(t(lang, "toolProfiles.toolNameLabel"))</label>
                <input type="text" name="tool_name">
                <label>(t(lang, "toolProfiles.riskLevelLabel"))</label>
                <input type="text" name="risk_level">
                <label>(t(lang, "toolProfiles.conditionsLabel"))</label>
                <input type="text" name="conditions" placeholder="null">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
            if entry_rows.is_empty() {
                <p class="empty">(t(lang, "toolProfiles.entriesNone"))</p>
            } else {
                <ul class="list">
                    for entry in entry_rows {
                        <li>
                            <span class="mono">(entry.profile_id.clone())</span>
                            " " <span class="badge badge-default">(entry.selector_type.clone())</span>
                            " " <span class="badge badge-default">(entry.effect.clone())</span>
                            " " <span class="meta-row">(entry.tool_name.clone().unwrap_or_default())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Tool connections page: connections and installs for one company.
#[page("/companies/{company_id}/tools/connections")]
pub async fn tool_connections(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let connection_rows = state
        .tool_connections
        .list_connections(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let application_rows = state
        .tool_catalog
        .list_applications(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let install_rows = state
        .tool_connections
        .list_installs(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "toolConnections.title"))</h1>
        <section>
            <h2>(t(lang, "toolConnections.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/tools/connections/ui"), lang))>
                <label>(t(lang, "toolConnections.applicationLabel"))</label>
                <select name="application_id">
                    for application in &application_rows {
                        <option value=(application.id.clone())>(application.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "toolConnections.nameLabel"))</label>
                <input type="text" name="name" required="required">
                <label>(t(lang, "toolConnections.uidLabel"))</label>
                <input type="text" name="uid" required="required">
                <label>(t(lang, "toolConnections.transportLabel"))</label>
                <select name="transport">
                    <option value="mcp_remote">"mcp_remote"</option>
                    <option value="rest_api">"rest_api"</option>
                    <option value="local_stdio">"local_stdio"</option>
                </select>
                <label>(t(lang, "toolConnections.connectionKindLabel"))</label>
                <input type="text" name="connection_kind" value="managed">
                <label>(t(lang, "toolConnections.ownershipLabel"))</label>
                <input type="text" name="ownership" value="customer">
                <label>(t(lang, "toolConnections.authKindLabel"))</label>
                <select name="auth_kind">
                    <option value="none">"none"</option>
                    <option value="oauth">"oauth"</option>
                    <option value="api_key">"api_key"</option>
                </select>
                <label>(t(lang, "toolConnections.statusLabel"))</label>
                <input type="text" name="status" value="draft">
                <label class="inline-label"><input type="checkbox" name="enabled" value="1"> (t(lang, "toolConnections.enabledLabel"))</label>
                <label>(t(lang, "toolConnections.configLabel"))</label>
                <input type="text" name="config" placeholder="{}">
                <label>(t(lang, "toolConnections.transportConfigLabel"))</label>
                <input type="text" name="transport_config" placeholder="{}">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "toolConnections.list"))</h2>
            if connection_rows.is_empty() {
                <p class="empty">(t(lang, "toolConnections.none"))</p>
            } else {
                <ul class="list">
                    for connection in &connection_rows {
                        <li>
                            <strong>(connection.name.clone())</strong>
                            " " <span class="mono">(connection.uid.clone())</span>
                            " " <span class="badge badge-default">(connection.transport.clone())</span>
                            " " <span class=(status_badge_class(&connection.status))>(connection.status.clone())</span>
                            if connection.enabled {
                                " " <span class="badge badge-done">(t(lang, "toolConnections.enabledLabel"))</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "toolConnections.installs"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/tools/installs/ui"), lang))>
                <label>(t(lang, "toolConnections.connectionLabel"))</label>
                <select name="connection_id">
                    for connection in &connection_rows {
                        <option value=(connection.id.clone())>(connection.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "toolConnections.targetTypeLabel"))</label>
                <input type="text" name="target_type" required="required">
                <label>(t(lang, "toolConnections.targetIdLabel"))</label>
                <input type="text" name="target_id" required="required">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
            if install_rows.is_empty() {
                <p class="empty">(t(lang, "toolConnections.installsNone"))</p>
            } else {
                <ul class="list">
                    for install in install_rows {
                        <li>
                            <span class="mono">(install.connection_id.clone())</span>
                            " " <span class="badge badge-default">(install.target_type.clone())</span>
                            " " <span class="meta-row">(install.target_id.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Tool gateways page: MCP gateways and their sessions for one company.
#[page("/companies/{company_id}/tools/gateways")]
pub async fn tool_gateways(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let gateway_rows = state
        .tool_gateway
        .list_gateways(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let profile_rows = state
        .tool_catalog
        .list_profiles(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let session_rows = state
        .tool_gateway
        .list_gateway_sessions(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "toolGateways.title"))</h1>
        <section>
            <h2>(t(lang, "toolGateways.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/tools/gateways/ui"), lang))>
                <label>(t(lang, "toolGateways.nameLabel"))</label>
                <input type="text" name="name" required="required">
                <label>(t(lang, "toolGateways.slugLabel"))</label>
                <input type="text" name="slug" required="required">
                <label>(t(lang, "toolGateways.displaySlugLabel"))</label>
                <input type="text" name="display_slug">
                <label>(t(lang, "toolGateways.descriptionLabel"))</label>
                <input type="text" name="description">
                <label>(t(lang, "toolGateways.statusLabel"))</label>
                <input type="text" name="status" value="active">
                <label>(t(lang, "toolGateways.profileLabel"))</label>
                <select name="profile_id">
                    for profile in &profile_rows {
                        <option value=(profile.id.clone())>(profile.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "toolGateways.defaultProfileModeLabel"))</label>
                <input type="text" name="default_profile_mode" value="gateway_only">
                <label>(t(lang, "toolGateways.contextScopeTypeLabel"))</label>
                <input type="text" name="context_scope_type" value="none">
                <label>(t(lang, "toolGateways.contextScopeIdLabel"))</label>
                <input type="text" name="context_scope_id">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "toolGateways.list"))</h2>
            if gateway_rows.is_empty() {
                <p class="empty">(t(lang, "toolGateways.none"))</p>
            } else {
                <ul class="list">
                    for gateway in &gateway_rows {
                        <li>
                            <strong>(gateway.name.clone())</strong>
                            " " <span class="mono">(gateway.slug.clone())</span>
                            " " <span class=(status_badge_class(&gateway.status))>(gateway.status.clone())</span>
                            " " <span class="badge badge-default">(gateway.default_profile_mode.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "toolGateways.sessions"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/tools/gateway-sessions/ui"), lang))>
                <label>(t(lang, "toolGateways.gatewayLabel"))</label>
                <select name="gateway_id">
                    for gateway in &gateway_rows {
                        <option value=(gateway.id.clone())>(gateway.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "toolGateways.agentLabel"))</label>
                <select name="agent_id">
                    for agent in &agent_rows {
                        <option value=(agent.id.clone())>(agent.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "toolGateways.runIdLabel"))</label>
                <input type="text" name="run_id" required="required">
                <label>(t(lang, "toolGateways.tokenHashLabel"))</label>
                <input type="text" name="token_hash" required="required">
                <label>(t(lang, "toolGateways.expiresAtLabel"))</label>
                <input type="text" name="expires_at" placeholder="2026-08-04T00:00:00.000Z" required="required">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
            if session_rows.is_empty() {
                <p class="empty">(t(lang, "toolGateways.sessionsNone"))</p>
            } else {
                <ul class="list">
                    for session in session_rows {
                        <li>
                            <span class="mono">(session.run_id.clone())</span>
                            " " <span class="badge badge-default">(session.agent_id.clone())</span>
                            " " <span class="meta-row">(session.gateway_public_id.clone().unwrap_or_default())</span>
                            " " <span class="meta-row">(session.expires_at.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Tool catalog page: discovered catalog entries for one company.
/// Apps aggregate page: applications + connections overview.
#[page("/companies/{company_id}/apps")]
pub async fn apps(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let applications = state
        .tool_catalog
        .list_applications(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let connections = state
        .tool_connections
        .list_connections(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let gates = state
        .tool_gateway
        .list_gateways(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "apps.title"))</h1>
        <nav class="nav-row">
            <a href=(with_lang(&format!("/companies/{company_id}/apps/browse"), lang))>(t(lang, "apps.browse"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/apps/gateways"), lang))>(t(lang, "apps.gateways"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/apps/advanced"), lang))>(t(lang, "apps.advanced"))</a>
        </nav>
        <section>
            <h2>(t(lang, "apps.connections"))</h2>
            if connections.is_empty() {
                <p class="empty">(t(lang, "apps.noConnections"))</p>
            } else {
                <ul class="list">
                    for connection in &connections {
                        <li>
                            <a href=(with_lang(&format!("/companies/{company_id}/apps/connections/{}", connection.id), lang))>
                                <strong>(connection.name.clone())</strong>
                            </a>
                            " " <span class="mono">(connection.transport.clone())</span>
                            " " <span class="badge badge-default">(connection.status.clone())</span>
                            " " <span class="meta-row">(connection.auth_kind.clone())</span>
                            " " <a class="button" href=(with_lang(&format!("/companies/{company_id}/tools/connections"), lang))>
                                (t(lang, "apps.manage"))
                            </a>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "apps.detail"))</h2>
            if applications.is_empty() {
                <p class="empty">(t(lang, "apps.empty"))</p>
            } else {
                <ul class="list">
                    for application in &applications {
                        <li>
                            <span class="mono">(application.r#type.clone())</span>
                            " " <strong>(application.name.clone())</strong>
                            " " <span class=(status_badge_class(&application.status))>(application.status.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "apps.gateways"))</h2>
            if gates.is_empty() {
                <p class="empty">(t(lang, "apps.noGateways"))</p>
            } else {
                <ul class="list">
                    for gateway in &gates {
                        <li>
                            <strong>(gateway.name.clone())</strong>
                            " " <span class="mono">(gateway.slug.clone())</span>
                            " " <span class=(status_badge_class(&gateway.status))>(gateway.status.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// App detail page: one connection's overview.
#[page("/companies/{company_id}/apps/connections/{connection_id}")]
pub async fn app_detail(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let connection_id = path_param::<ConnectionId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let connection = state
        .tool_connections
        .get_connection(&company_id, &connection_id)
        .await
        .map_err(to_topcoat_error)?
        .ok_or_else(topcoat::router::error::not_found)?;
    let grants = state
        .tool_connections
        .list_grants(&company_id, Some(&connection_id))
        .await
        .map_err(to_topcoat_error)?;
    let installs = state
        .tool_connections
        .list_installs(&company_id, Some(&connection_id))
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "apps.detail")) ": " (connection.name.clone())</h1>
        <p class="meta-row">(connection.id.clone())</p>
        <p class="meta-row">(connection.uid.clone())</p>
        <p class="meta-row">(connection.transport.clone()) " / " (connection.auth_kind.clone())</p>
        <p class="meta-row">
            (t(lang, "apps.connections")) ": " <span class=(status_badge_class(&connection.status))>(connection.status.clone())</span>
        </p>
        <section>
            <h2>(t(lang, "apps.grants"))</h2>
            if grants.is_empty() {
                <p class="empty">(t(lang, "apps.noGrants"))</p>
            } else {
                <ul class="list">
                    for grant in &grants {
                        <li>
                            <span class="mono">(grant.kind.clone())</span>
                            " " <span class="meta-row">(grant.subject_user_id.clone().unwrap_or_default())</span>
                            " " <span class=(status_badge_class(&grant.status))>(grant.status.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <section>
            <h2>(t(lang, "toolConnections.installs"))</h2>
            if installs.is_empty() {
                <p class="empty">(t(lang, "toolConnections.installsNone"))</p>
            } else {
                <ul class="list">
                    for install in &installs {
                        <li>
                            <span class="mono">(install.target_type.clone())</span>
                            " " <span class="meta-row">(install.target_id.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
        <a class="button" href=(with_lang(&format!("/companies/{company_id}/tools/connections"), lang))>
            (t(lang, "apps.manage"))
        </a>
    }
}

/// App browse page: catalog entries.
#[page("/companies/{company_id}/apps/browse")]
pub async fn apps_browse(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let entries = state
        .tool_catalog
        .list_catalog_entries(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "apps.browse"))</h1>
        if entries.is_empty() {
            <p class="empty">(t(lang, "apps.noCatalogEntries"))</p>
        } else {
            <ul class="list">
                for entry in &entries {
                    <li>
                        <span class="mono">(entry.entry_kind.clone())</span>
                        " " <strong>(entry.name.clone())</strong>
                        " " <span class="badge badge-default">(entry.status.clone())</span>
                        if let Some(title) = &entry.title {
                            " " <span class="meta-row">(title.clone())</span>
                        }
                    </li>
                }
            </ul>
        }
        <a class="button" href=(with_lang(&format!("/companies/{company_id}/tools/catalog"), lang))>
            (t(lang, "apps.manage"))
        </a>
    }
}

/// Apps gateways page.
#[page("/companies/{company_id}/apps/gateways")]
pub async fn apps_gateways(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let gates = state
        .tool_gateway
        .list_gateways(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "apps.gateways"))</h1>
        if gates.is_empty() {
            <p class="empty">(t(lang, "apps.noGateways"))</p>
        } else {
            <ul class="list">
                for gateway in &gates {
                    <li>
                        <strong>(gateway.name.clone())</strong>
                        " " <span class="mono">(gateway.slug.clone())</span>
                        " " <span class=(status_badge_class(&gateway.status))>(gateway.status.clone())</span>
                    </li>
                }
            </ul>
        }
        <a class="button" href=(with_lang(&format!("/companies/{company_id}/tools/gateways"), lang))>
            (t(lang, "apps.manage"))
        </a>
    }
}

/// Apps advanced tools page: profiles/invocations/audit entry points.
#[page("/companies/{company_id}/apps/advanced")]
pub async fn apps_advanced(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    view! {
        <h1 class="page-title">(t(lang, "apps.advancedTools"))</h1>
        <nav class="nav-row">
            <a href=(with_lang(&format!("/companies/{company_id}/tools/profiles"), lang))>(t(lang, "apps.profiles"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/tools/invocations"), lang))>(t(lang, "apps.invocations"))</a>
            <a href=(with_lang(&format!("/companies/{company_id}/tools/audit-events"), lang))>(t(lang, "apps.audit"))</a>
        </nav>
    }
}

#[page("/companies/{company_id}/tools/catalog")]
pub async fn tool_catalog(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let catalog_rows = state
        .tool_catalog
        .list_catalog_entries(&company_id, None)
        .await
        .map_err(to_topcoat_error)?;
    let connection_rows = state
        .tool_connections
        .list_connections(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "toolCatalog.title"))</h1>
        <section>
            <h2>(t(lang, "toolCatalog.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/tools/catalog/ui"), lang))>
                <label>(t(lang, "toolCatalog.connectionLabel"))</label>
                <select name="connection_id">
                    for connection in &connection_rows {
                        <option value=(connection.id.clone())>(connection.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "toolCatalog.entryKindLabel"))</label>
                <input type="text" name="entry_kind" value="tool">
                <label>(t(lang, "toolCatalog.nameLabel"))</label>
                <input type="text" name="name" required="required">
                <label>(t(lang, "toolCatalog.toolNameLabel"))</label>
                <input type="text" name="tool_name" required="required">
                <label>(t(lang, "toolCatalog.titleLabel"))</label>
                <input type="text" name="title">
                <label>(t(lang, "toolCatalog.descriptionLabel"))</label>
                <input type="text" name="description">
                <label>(t(lang, "toolCatalog.riskLevelLabel"))</label>
                <input type="text" name="risk_level" value="medium">
                <label>(t(lang, "toolCatalog.statusLabel"))</label>
                <input type="text" name="status" value="active">
                <label>(t(lang, "toolCatalog.versionLabel"))</label>
                <input type="text" name="version">
                <label>(t(lang, "toolCatalog.inputSchemaLabel"))</label>
                <input type="text" name="input_schema" placeholder="{}">
                <label class="inline-label"><input type="checkbox" name="is_read_only" value="1"> (t(lang, "toolCatalog.readOnlyLabel"))</label>
                <label class="inline-label"><input type="checkbox" name="is_write" value="1"> (t(lang, "toolCatalog.writeLabel"))</label>
                <label class="inline-label"><input type="checkbox" name="is_destructive" value="1"> (t(lang, "toolCatalog.destructiveLabel"))</label>
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "toolCatalog.list"))</h2>
            if catalog_rows.is_empty() {
                <p class="empty">(t(lang, "toolCatalog.none"))</p>
            } else {
                <ul class="list">
                    for entry in catalog_rows {
                        <li>
                            <strong>(entry.name.clone())</strong>
                            " " <span class="mono">(entry.tool_name.clone())</span>
                            " " <span class="badge badge-default">(entry.entry_kind.clone())</span>
                            " " <span class="badge badge-default">(entry.risk_level.clone())</span>
                            " " <span class=(status_badge_class(&entry.status))>(entry.status)</span>
                            if entry.is_destructive {
                                " " <span class="badge badge-blocked">(t(lang, "toolCatalog.destructiveLabel"))</span>
                            }
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// Tool invocations page: recorded tool calls for one company.
#[page("/companies/{company_id}/tools/invocations")]
pub async fn tool_invocations(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let invocation_rows = state
        .tool_gateway
        .list_invocations(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let connection_rows = state
        .tool_connections
        .list_connections(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    let agent_rows = state
        .agents
        .list(&company_id)
        .await
        .map_err(to_topcoat_error)?;
    view! {
        <h1 class="page-title">(t(lang, "toolInvocations.title"))</h1>
        <section>
            <h2>(t(lang, "toolInvocations.create"))</h2>
            <form class="stack-form" method="post"
                  action=(with_lang(&format!("/companies/{company_id}/tools/invocations/ui"), lang))>
                <label>(t(lang, "toolInvocations.toolNameLabel"))</label>
                <input type="text" name="tool_name" required="required">
                <label>(t(lang, "toolInvocations.actorTypeLabel"))</label>
                <input type="text" name="actor_type" value="system">
                <label>(t(lang, "toolInvocations.statusLabel"))</label>
                <input type="text" name="status" value="pending">
                <label>(t(lang, "toolInvocations.approvalStateLabel"))</label>
                <input type="text" name="approval_state" value="not_required">
                <label>(t(lang, "toolInvocations.riskLevelLabel"))</label>
                <input type="text" name="risk_level">
                <label>(t(lang, "toolInvocations.connectionLabel"))</label>
                <select name="connection_id">
                    <option value="">(t(lang, "common.none"))</option>
                    for connection in &connection_rows {
                        <option value=(connection.id.clone())>(connection.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "toolInvocations.agentLabel"))</label>
                <select name="agent_id">
                    <option value="">(t(lang, "common.none"))</option>
                    for agent in &agent_rows {
                        <option value=(agent.id.clone())>(agent.name.clone())</option>
                    }
                </select>
                <label>(t(lang, "toolInvocations.argumentsSummaryLabel"))</label>
                <input type="text" name="arguments_summary" placeholder="null">
                <button type="submit">(t(lang, "common.create"))</button>
            </form>
        </section>
        <section>
            <h2>(t(lang, "toolInvocations.list"))</h2>
            if invocation_rows.is_empty() {
                <p class="empty">(t(lang, "toolInvocations.none"))</p>
            } else {
                <ul class="list">
                    for invocation in invocation_rows {
                        <li>
                            <strong>(invocation.tool_name.clone())</strong>
                            " " <span class="badge badge-default">(invocation.actor_type.clone())</span>
                            " " <span class=(status_badge_class(&invocation.status))>(invocation.status)</span>
                            " " <span class="meta-row">(invocation.approval_state.clone())</span>
                            " " <span class="meta-row">(invocation.created_at.clone())</span>
                        </li>
                    }
                </ul>
            }
        </section>
    }
}

/// `{pipeline_id}` path parameter for UI pages.
#[path_param(error = bad_request("Invalid pipeline id"))]
pub(crate) struct PipelineId(String);

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

/// Friendly not-found page for unknown paths. API paths keep the router's
/// JSON 404 response so API clients see the same error shape as before.
#[page("/{*path}")]
pub async fn not_found(cx: &Cx) -> Result {
    let lang = lang_from_request(cx);
    let parts = topcoat::context::try_request_context::<http::request::Parts>(cx);
    let current_path = parts
        .map(|parts| parts.uri.path().to_owned())
        .unwrap_or_else(|| "/".to_owned());
    if current_path.starts_with("/api/") {
        return Err(topcoat::router::error::not_found().into());
    }
    view! {
        <h1 class="page-title">(t(lang, "notFound.title"))</h1>
        <p class="meta-row">(t(lang, "notFound.message"))</p>
        <p class="mono">(t(lang, "notFound.requestedPath")) ": " (current_path)</p>
        <nav class="nav-row">
            <a href=(with_lang("/", lang))>(t(lang, "notFound.goHome"))</a>
            <a href=(with_lang("/instance/settings", lang))>(t(lang, "instance.title"))</a>
        </nav>
    }
}
