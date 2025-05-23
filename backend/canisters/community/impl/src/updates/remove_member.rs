use crate::{
    RuntimeState, activity_notifications::handle_activity_notification, model::events::CommunityEventInternal, mutate_state,
    read_state, run_regular_jobs,
};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use community_canister::remove_member::*;
use fire_and_forget_handler::FireAndForgetHandler;
use local_user_index_canister_c2c_client::lookup_user;
use msgpack::serialize_then_unwrap;
use oc_error_codes::OCErrorCode;
use std::collections::HashMap;
use types::{CanisterId, CommunityMembersRemoved, CommunityRole, CommunityUsersBlocked, OCResult, UserId};
use user_canister::c2c_remove_from_community;

#[update(msgpack = true)]
#[trace]
async fn block_user(args: community_canister::block_user::Args) -> community_canister::block_user::Response {
    run_regular_jobs();

    if !read_state(|state| state.data.is_public.value) {
        return Response::Error(OCErrorCode::CommunityNotPublic.into());
    }

    remove_member_impl(args.user_id, true).await
}

#[update(msgpack = true)]
#[trace]
async fn remove_member(args: Args) -> Response {
    run_regular_jobs();

    remove_member_impl(args.user_id, false).await
}

async fn remove_member_impl(user_id: UserId, block: bool) -> Response {
    // Check the caller can remove the user
    let prepare_result = match read_state(|state| prepare(user_id, block, state)) {
        Ok(ok) => ok,
        Err(error) => return Response::Error(error),
    };

    // If the user is an owner of the community then call the local_user_index
    // to check whether they are a "platform moderator" in which case this removal
    // is not authorized
    if prepare_result.is_user_to_remove_an_owner {
        match lookup_user(user_id.into(), prepare_result.local_user_index_canister_id).await {
            Ok(Some(user)) if !user.is_platform_moderator => (),
            Ok(_) => return Response::Error(OCErrorCode::InitiatorNotAuthorized.into()),
            Err(error) => return Response::Error(error.into()),
        }
    }

    // Remove the user from the community
    mutate_state(|state| commit(user_id, block, prepare_result.removed_by, state));

    Response::Success
}

struct PrepareResult {
    removed_by: UserId,
    local_user_index_canister_id: CanisterId,
    is_user_to_remove_an_owner: bool,
}

fn prepare(user_id: UserId, block: bool, state: &RuntimeState) -> OCResult<PrepareResult> {
    state.data.verify_not_frozen()?;

    let member = state.get_calling_member(true)?;
    if member.user_id == user_id {
        Err(OCErrorCode::CannotRemoveSelf.into())
    } else {
        let user_to_remove_role = match state.data.members.get_by_user_id(&user_id) {
            Some(member_to_remove) => member_to_remove.role(),
            None if block => {
                if state.data.members.is_blocked(&user_id) {
                    return Err(OCErrorCode::NoChange.into());
                }
                CommunityRole::Member
            }
            None => return Err(OCErrorCode::TargetUserNotInCommunity.into()),
        };

        // Check if the caller is authorized to remove the user
        if member
            .role()
            .can_remove_members_with_role(user_to_remove_role, &state.data.permissions)
        {
            Ok(PrepareResult {
                removed_by: member.user_id,
                local_user_index_canister_id: state.data.local_user_index_canister_id,
                is_user_to_remove_an_owner: user_to_remove_role.is_owner(),
            })
        } else {
            Err(OCErrorCode::InitiatorNotAuthorized.into())
        }
    }
}

fn commit(user_id: UserId, block: bool, removed_by: UserId, state: &mut RuntimeState) {
    let now = state.env.now();

    // Remove the user from the community
    let removed_member = state.data.remove_user_from_community(user_id, None, now);
    let removed = removed_member.is_some();

    let blocked = block && state.data.members.block(user_id, now);

    let referred_by = removed_member
        .and_then(|r| r.referred_by)
        .map_or(HashMap::new(), |referred_by| HashMap::from_iter([(user_id, referred_by)]));

    // Push relevant event
    let event = if blocked {
        let event = CommunityUsersBlocked {
            user_ids: vec![user_id],
            blocked_by: removed_by,
            referred_by,
        };
        CommunityEventInternal::UsersBlocked(Box::new(event))
    } else if removed {
        let event = CommunityMembersRemoved {
            user_ids: vec![user_id],
            removed_by,
            referred_by,
        };
        CommunityEventInternal::MembersRemoved(Box::new(event))
    } else {
        return;
    };
    state.data.events.push_event(event, now);

    handle_activity_notification(state);

    if removed {
        // Fire-and-forget call to notify the user canister
        remove_membership_from_user_canister(
            user_id,
            removed_by,
            block,
            state.data.name.value.clone(),
            state.data.is_public.value,
            &mut state.data.fire_and_forget_handler,
        );
    }
}

fn remove_membership_from_user_canister(
    user_id: UserId,
    removed_by: UserId,
    blocked: bool,
    community_name: String,
    public: bool,
    fire_and_forget_handler: &mut FireAndForgetHandler,
) {
    let args = c2c_remove_from_community::Args {
        removed_by,
        blocked,
        community_name,
        public,
    };
    fire_and_forget_handler.send(
        user_id.into(),
        "c2c_remove_from_community_msgpack".to_string(),
        serialize_then_unwrap(args),
    );
}
