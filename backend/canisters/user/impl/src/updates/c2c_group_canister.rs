use crate::guards::caller_is_known_group_canister;
use crate::{RuntimeState, mutate_state, run_regular_jobs};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use user_canister::GroupCanisterEvent;
use user_canister::c2c_group_canister::{Response::*, *};

#[update(guard = "caller_is_known_group_canister", msgpack = true)]
#[trace]
fn c2c_group_canister(args: Args) -> Response {
    run_regular_jobs();

    mutate_state(|state| c2c_notify_group_canister_events_impl(args, state))
}

fn c2c_notify_group_canister_events_impl(args: Args, state: &mut RuntimeState) -> Response {
    let caller = state.env.caller();
    let now = state.env.now();
    let mut awarded_achievement = false;

    for event in args.events {
        if state
            .data
            .idempotency_checker
            .check(caller, event.created_at, event.idempotency_id)
        {
            match event.value {
                GroupCanisterEvent::MessageActivity(event) => state.data.push_message_activity(event, now),
                GroupCanisterEvent::Achievement(achievement) => {
                    awarded_achievement |= state.data.award_achievement(achievement, now);
                }
            }
        }
    }

    if awarded_achievement {
        state.notify_user_index_of_chit(now);
    }

    Success
}
