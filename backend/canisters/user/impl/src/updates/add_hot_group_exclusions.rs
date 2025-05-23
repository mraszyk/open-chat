use crate::guards::caller_is_owner;
use crate::{RuntimeState, mutate_state, run_regular_jobs};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use user_canister::add_hot_group_exclusions::{Response::*, *};

#[update(guard = "caller_is_owner", msgpack = true)]
#[trace]
fn add_hot_group_exclusions(args: Args) -> Response {
    run_regular_jobs();

    mutate_state(|state| add_hot_group_exclusions_impl(args, state))
}

#[update(guard = "caller_is_owner", msgpack = true)]
#[trace]
fn add_recommended_group_exclusions(args: Args) -> Response {
    run_regular_jobs();

    mutate_state(|state| add_hot_group_exclusions_impl(args, state))
}

fn add_hot_group_exclusions_impl(args: Args, state: &mut RuntimeState) -> Response {
    let now = state.env.now();
    for group in args.groups {
        state.data.hot_group_exclusions.add(group, args.duration, now);
    }
    Success
}
