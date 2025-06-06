use crate::{RuntimeState, mutate_state, run_regular_jobs};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use group_canister::toggle_mute_notifications::*;
use types::OCResult;

#[update(msgpack = true)]
#[trace]
fn toggle_mute_notifications(args: Args) -> Response {
    run_regular_jobs();

    mutate_state(|state| toggle_mute_notifications_impl(args, state)).into()
}

fn toggle_mute_notifications_impl(args: Args, state: &mut RuntimeState) -> OCResult {
    let user_id = state.get_caller_user_id()?;
    let now = state.env.now();
    if matches!(
        state.data.chat.members.toggle_notifications_muted(user_id, args.mute, now),
        Some(true)
    ) {
        state.data.mark_group_updated_in_user_canister(user_id);
    }
    Ok(())
}
