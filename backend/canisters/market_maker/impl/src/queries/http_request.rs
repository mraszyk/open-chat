use crate::{RuntimeState, read_state};
use http_request::{Route, build_json_response, build_response, encode_logs, extract_route};
use ic_cdk::query;
use std::io::Write;
use types::{HttpRequest, HttpResponse, TimestampMillis};

#[query]
fn http_request(request: HttpRequest) -> HttpResponse {
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

    fn get_order_logs(state: &RuntimeState) -> HttpResponse {
        let mut body = Vec::new();

        let skip = state.data.orders_log.len().saturating_sub(200);

        for log in state.data.orders_log.iter().skip(skip as usize) {
            writeln!(&mut body, "{log}").unwrap();
        }

        build_response(body, "text/plain")
    }

    fn get_balance_history(state: &RuntimeState) -> HttpResponse {
        build_json_response(&state.data.balance_history.iter().take(5000).collect::<Vec<_>>())
    }

    match extract_route(&request.url) {
        Route::Errors(since) => get_errors_impl(since),
        Route::Logs(since) => get_logs_impl(since),
        Route::Traces(since) => get_traces_impl(since),
        Route::Metrics => read_state(get_metrics_impl),
        Route::Other(p, _) if p == "orders" => read_state(get_order_logs),
        Route::Other(p, _) if p == "balance_history" => read_state(get_balance_history),
        _ => HttpResponse::not_found(),
    }
}
