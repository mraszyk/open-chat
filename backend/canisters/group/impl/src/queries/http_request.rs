use crate::{RuntimeState, read_state};
use http_request::{AvatarRoute, Route, build_json_response, encode_logs, extract_route, get_document};
use ic_cdk::query;
use types::{HttpRequest, HttpResponse, TimestampMillis};

#[query]
fn http_request(request: HttpRequest) -> HttpResponse {
    fn get_avatar_impl(route: AvatarRoute, state: &RuntimeState) -> HttpResponse {
        let (avatar, path) = if let Some(id) = route.bot_id {
            (
                state.data.webhooks.get(&id).and_then(|w| w.avatar.as_ref()),
                format!("avatar/{id}"),
            )
        } else {
            (state.data.chat.avatar.as_ref(), "avatar".to_string())
        };

        get_document(route.blob_id, avatar, &path)
    }

    fn get_errors_impl(since: Option<TimestampMillis>) -> HttpResponse {
        encode_logs(canister_logger::export_errors(), since.unwrap_or(0))
    }

    fn get_logs_impl(since: Option<TimestampMillis>) -> HttpResponse {
        encode_logs(canister_logger::export_logs(), since.unwrap_or(0))
    }

    fn get_traces_impl(since: Option<TimestampMillis>) -> HttpResponse {
        encode_logs(canister_logger::export_traces(), since.unwrap_or(0))
    }

    fn get_metrics_impl(state: &RuntimeState) -> HttpResponse {
        build_json_response(&state.metrics())
    }

    fn get_timer_jobs(state: &RuntimeState) -> HttpResponse {
        let data: Vec<_> = if state.data.chat.is_public.value || state.data.test_mode {
            state
                .data
                .timer_jobs
                .iter()
                .filter_map(|(ts, wrapper)| wrapper.borrow().as_ref().map(|j| (*ts, j.clone())))
                .collect()
        } else {
            Vec::new()
        };

        build_json_response(&data)
    }

    match extract_route(&request.url) {
        Route::Avatar(route) => read_state(|state| get_avatar_impl(route, state)),
        Route::Errors(since) => get_errors_impl(since),
        Route::Logs(since) => get_logs_impl(since),
        Route::Traces(since) => get_traces_impl(since),
        Route::Metrics => read_state(get_metrics_impl),
        Route::Other(p, _) if p == "timer_jobs" => read_state(get_timer_jobs),
        Route::Webhook(_) if request.method.eq_ignore_ascii_case("POST") => HttpResponse::upgrade(),
        _ => HttpResponse::not_found(),
    }
}
