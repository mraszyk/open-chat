use crate::guards::caller_is_push_service;
use crate::{RuntimeState, read_state};
use ic_cdk::query;
use notifications_canister::latest_notification_index::{Response::*, *};

#[query(guard = "caller_is_push_service")]
fn latest_notification_index(_args: Args) -> Response {
    read_state(latest_notification_index_impl)
}

fn latest_notification_index_impl(state: &RuntimeState) -> Response {
    Success(state.data.notifications.latest_event_index())
}
