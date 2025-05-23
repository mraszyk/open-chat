use crate::guards::caller_is_local_user_index;
use crate::{RuntimeState, activity_notifications::handle_activity_notification, mutate_state, run_regular_jobs};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use oc_error_codes::OCErrorCode;
use types::c2c_install_bot::*;
use types::{BotGroupConfig, OCResult};

#[update(guard = "caller_is_local_user_index", msgpack = true)]
#[trace]
fn c2c_install_bot(args: Args) -> Response {
    run_regular_jobs();

    mutate_state(|state| c2c_install_bot_impl(args, state)).into()
}

fn c2c_install_bot_impl(args: Args, state: &mut RuntimeState) -> OCResult {
    state.data.verify_not_frozen()?;

    let member = state.data.members.get_verified_member(args.caller.into())?;

    if !member.role().is_owner() {
        return Err(OCErrorCode::InitiatorNotAuthorized.into());
    }

    if !state.data.install_bot(
        member.user_id,
        args.bot_id,
        BotGroupConfig {
            permissions: args.granted_permissions,
        },
        state.env.now(),
    ) {
        return Err(OCErrorCode::AlreadyAdded.into());
    }

    handle_activity_notification(state);
    Ok(())
}
