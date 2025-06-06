use crate::guards::caller_is_authorized_to_add_canister;
use crate::{State, mutate_state};
use canister_api_macros::proposal;
use canister_tracing_macros::trace;
use cycles_dispenser_canister::add_canister::{Response::*, *};

#[proposal(guard = "caller_is_authorized_to_add_canister")]
#[trace]
fn add_canister(args: Args) -> Response {
    mutate_state(|state| add_canister_impl(args, state))
}

fn add_canister_impl(args: Args, state: &mut State) -> Response {
    let now = state.env.now();
    if state.data.canisters.add(args.canister_id, now) { Success } else { AlreadyAdded }
}
