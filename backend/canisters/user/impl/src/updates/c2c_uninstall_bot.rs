use crate::guards::caller_is_local_user_index;
use crate::{RuntimeState, mutate_state, run_regular_jobs};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use constants::OPENCHAT_BOT_USER_ID;
use oc_error_codes::OCErrorCode;
use types::OCResult;
use types::c2c_uninstall_bot::*;

#[update(guard = "caller_is_local_user_index", msgpack = true)]
#[trace]
fn c2c_uninstall_bot(args: Args) -> Response {
    run_regular_jobs();

    mutate_state(|state| c2c_uninstall_bot_impl(args, state)).into()
}

fn c2c_uninstall_bot_impl(args: Args, state: &mut RuntimeState) -> OCResult {
    if args.caller != OPENCHAT_BOT_USER_ID {
        if args.caller != state.env.canister_id().into() {
            return Err(OCErrorCode::InitiatorNotAuthorized.into());
        };

        if state.data.suspended.value {
            return Err(OCErrorCode::InitiatorSuspended.into());
        }
    }

    let now = state.env.now();

    state.data.bots.remove(args.bot_id, now);
    state.data.bot_api_keys.delete(args.bot_id);
    state.delete_direct_chat(args.bot_id, false, now);
    Ok(())
}
