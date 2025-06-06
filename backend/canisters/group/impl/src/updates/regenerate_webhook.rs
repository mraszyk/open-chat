use crate::{RuntimeState, activity_notifications::handle_activity_notification, mutate_state, run_regular_jobs};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use group_canister::regenerate_webhook::*;
use oc_error_codes::OCErrorCode;
use types::OCResult;

#[update(candid = true, msgpack = true)]
#[trace]
fn regenerate_webhook(args: Args) -> Response {
    run_regular_jobs();

    mutate_state(|state| regenerate_webhook_impl(args, state)).into()
}

fn regenerate_webhook_impl(args: Args, state: &mut RuntimeState) -> OCResult {
    state.data.verify_not_frozen()?;

    let member = state.get_calling_member(true)?;

    if !member.role().is_owner() {
        return Err(OCErrorCode::InitiatorNotAuthorized.into());
    }

    let now = state.env.now();

    if !state.data.webhooks.regenerate(args.id, state.env.rng(), now) {
        return Err(OCErrorCode::WebhookNotFound.into());
    }

    handle_activity_notification(state);
    Ok(())
}
