use crate::activity_notifications::handle_activity_notification;
use crate::{RuntimeState, mutate_state, run_regular_jobs};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use group_canister::unpin_message::{Response::*, *};
use types::{OCResult, PushEventResult};

#[update(msgpack = true)]
#[trace]
async fn unpin_message(args: Args) -> Response {
    run_regular_jobs();

    match mutate_state(|state| unpin_message_impl(args, state)) {
        Ok(result) => SuccessV2(result),
        Err(error) => Error(error),
    }
}

fn unpin_message_impl(args: Args, state: &mut RuntimeState) -> OCResult<PushEventResult> {
    state.data.verify_not_frozen()?;

    let user_id = state.get_caller_user_id()?;
    let now = state.env.now();
    let result = state.data.chat.unpin_message(user_id, args.message_index, now)?;
    handle_activity_notification(state);
    Ok(result.into())
}
