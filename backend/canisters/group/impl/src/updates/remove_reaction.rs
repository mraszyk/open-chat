use crate::{RuntimeState, activity_notifications::handle_activity_notification, mutate_state, run_regular_jobs};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use group_canister::remove_reaction::*;
use types::OCResult;

#[update(candid = true, msgpack = true)]
#[trace]
fn remove_reaction(args: Args) -> Response {
    run_regular_jobs();

    mutate_state(|state| remove_reaction_impl(args, state)).into()
}

fn remove_reaction_impl(args: Args, state: &mut RuntimeState) -> OCResult {
    state.data.verify_not_frozen()?;

    let user_id = state.get_caller_user_id()?;
    let now = state.env.now();

    state
        .data
        .chat
        .remove_reaction(user_id, args.thread_root_message_index, args.message_id, args.reaction, now)?;

    handle_activity_notification(state);
    Ok(())
}
