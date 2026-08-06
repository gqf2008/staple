//! Issue-based attention feed routes (upstream `routes/attention.ts` parity).

use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Json, path_param, route},
};

use crate::{
    attention::{AttentionQuery, build_attention_feed},
    auth::{enforce_company_scope, require_board},
    error::ApiError,
    routes::CompanyId,
    state::AppState,
};

fn query_param(cx: &Cx, name: &str) -> Option<String> {
    topcoat::context::try_request_context::<http::request::Parts>(cx).and_then(|parts| {
        parts.uri.query().and_then(|query| {
            query.split('&').find_map(|pair| {
                let (key, value) = pair.split_once('=')?;
                (key == name).then(|| value.to_owned())
            })
        })
    })
}

fn header_value(cx: &Cx, name: &str) -> Option<String> {
    topcoat::context::try_request_context::<http::request::Parts>(cx)
        .and_then(|parts| parts.headers.get(name))
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

/// `GET /api/companies/{companyId}/attention` — issue-based attention feed.
#[route(GET "/api/companies/{company_id}/attention")]
pub async fn company_attention(cx: &Cx) -> Result<Json<crate::attention::AttentionFeed>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    require_board(cx)?;
    let limit = query_param(cx, "limit")
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(50)
        .min(100);
    let cursor = query_param(cx, "cursor");
    let sort = query_param(cx, "sort").unwrap_or_else(|| "activity".to_owned());
    let include_dismissed = query_param(cx, "includeDismissed").as_deref() == Some("true");
    let activity_since = query_param(cx, "activitySince");
    let activity_until = query_param(cx, "activityUntil");
    let all = query_param(cx, "all").as_deref() == Some("true");
    let queue = query_param(cx, "queue");
    let user_id = header_value(cx, "x-board-user").unwrap_or_else(|| "board".to_owned());
    let state = app_context::<AppState>(cx);
    let feed = build_attention_feed(
        state,
        &company_id,
        &AttentionQuery {
            limit,
            cursor,
            sort,
            include_dismissed,
            activity_since,
            activity_until,
            all,
            queue,
            user_id,
        },
    )
    .await?;
    Ok(Json(feed))
}
