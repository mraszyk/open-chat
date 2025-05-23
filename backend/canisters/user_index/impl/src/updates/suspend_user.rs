use crate::guards::caller_is_platform_moderator;
use crate::timer_job_types::{SetUserSuspendedInCommunity, SetUserSuspendedInGroup, TimerJob, UnsuspendUser};
use crate::{RuntimeState, mutate_state, read_state};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use local_user_index_canister::{UserIndexEvent, UserSuspended};
use types::{ChatId, CommunityId, Milliseconds, SuspensionDuration, UserId};
use user_index_canister::suspend_user::{Response::*, *};

#[update(guard = "caller_is_platform_moderator", msgpack = true)]
#[trace]
async fn suspend_user(args: Args) -> Response {
    let suspended_by = match read_state(|state| prepare(&args.user_id, state)) {
        Err(response) => return response,
        Ok(ok) => ok,
    };

    suspend_user_impl(args.user_id, args.duration, args.reason, suspended_by).await
}

pub(crate) async fn suspend_user_impl(
    user_id: UserId,
    duration: Option<Milliseconds>,
    reason: String,
    suspended_by: UserId,
) -> Response {
    let c2c_args = user_canister::c2c_set_user_suspended::Args { suspended: true };
    match user_canister_c2c_client::c2c_set_user_suspended(user_id.into(), &c2c_args).await {
        Ok(user_canister::c2c_set_user_suspended::Response::Success(result)) => {
            mutate_state(|state| {
                commit(
                    user_id,
                    duration,
                    reason,
                    result.groups,
                    result.communities,
                    suspended_by,
                    state,
                )
            });
            Success
        }
        Err(error) => InternalError(format!("{error:?}")),
    }
}

fn prepare(user_id: &UserId, state: &RuntimeState) -> Result<UserId, Response> {
    match state.data.users.is_user_suspended(user_id) {
        Some(false) => {
            let caller = state.env.caller();
            Ok(state.data.users.get_by_principal(&caller).unwrap().user_id)
        }
        Some(true) => Err(UserAlreadySuspended),
        None => Err(UserNotFound),
    }
}

fn commit(
    user_id: UserId,
    duration: Option<Milliseconds>,
    reason: String,
    groups: Vec<ChatId>,
    communities: Vec<CommunityId>,
    suspended_by: UserId,
    state: &mut RuntimeState,
) {
    let now = state.env.now();
    for group in groups {
        state.data.timer_jobs.enqueue_job(
            TimerJob::SetUserSuspendedInGroup(SetUserSuspendedInGroup {
                user_id,
                group,
                suspended: true,
                attempt: 0,
            }),
            now,
            now,
        );
    }

    for community in communities {
        state.data.timer_jobs.enqueue_job(
            TimerJob::SetUserSuspendedInCommunity(SetUserSuspendedInCommunity {
                user_id,
                community,
                suspended: true,
                attempt: 0,
            }),
            now,
            now,
        );
    }

    state
        .data
        .users
        .suspend_user(user_id, duration, reason.clone(), suspended_by, now);

    // If the user is only suspended for a specified duration, schedule them to be unsuspended
    if let Some(ms) = duration {
        state
            .data
            .timer_jobs
            .enqueue_job(TimerJob::UnsuspendUser(UnsuspendUser { user_id }), now + ms, now);
    }

    state.push_event_to_local_user_index(
        user_id,
        UserIndexEvent::UserSuspended(UserSuspended {
            user_id,
            timestamp: now,
            duration: duration.map_or(SuspensionDuration::Indefinitely, SuspensionDuration::Duration),
            reason,
            suspended_by,
        }),
    );
}
