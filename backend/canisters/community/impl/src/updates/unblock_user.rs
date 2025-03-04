use crate::activity_notifications::handle_activity_notification;
use crate::model::events::CommunityEventInternal;
use crate::updates::unblock_user::Response::*;
use crate::{mutate_state, run_regular_jobs, RuntimeState};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use community_canister::unblock_user::*;
use types::UsersUnblocked;

#[update(msgpack = true)]
#[trace]
fn unblock_user(args: Args) -> Response {
    run_regular_jobs();

    mutate_state(|state| unblock_user_impl(args, state))
}

fn unblock_user_impl(args: Args, state: &mut RuntimeState) -> Response {
    if state.data.is_frozen() {
        return CommunityFrozen;
    }

    let caller = state.env.caller();

    if !state.data.is_public.value {
        CommunityNotPublic
    } else if let Some(caller_member) = state.data.members.get(caller) {
        if caller_member.suspended().value {
            return UserSuspended;
        } else if caller_member.lapsed().value {
            return UserLapsed;
        }

        let unblocked_by = caller_member.user_id;
        if unblocked_by == args.user_id {
            CannotUnblockSelf
        } else if caller_member.role().can_unblock_users(&state.data.permissions) {
            let now = state.env.now();

            state.data.members.unblock(args.user_id, now);

            let event = UsersUnblocked {
                user_ids: vec![args.user_id],
                unblocked_by,
            };

            state
                .data
                .events
                .push_event(CommunityEventInternal::UsersUnblocked(Box::new(event)), now);

            handle_activity_notification(state);

            Success
        } else {
            NotAuthorized
        }
    } else {
        UserNotInCommunity
    }
}
