use crate::{State, mutate_state};
use canister_tracing_macros::trace;
use cycles_dispenser_canister::c2c_request_cycles::{Response::*, *};
use ic_cdk::update;
use std::cmp::min;
use tracing::info;
use types::{CanisterId, Cycles, Milliseconds, TimestampMillis};
use utils::canister::deposit_cycles;

#[update]
#[trace]
async fn c2c_request_cycles(args: Args) -> Response {
    let PrepareResult { canister_id, amount } = match mutate_state(|state| prepare(args, state)) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let result = deposit_cycles(canister_id, amount).await;

    mutate_state(|state| commit(&canister_id, result.is_ok().then_some(amount), state));

    match result {
        Ok(_) => Success(amount),
        Err(e) => InternalError(format!("{e:?}")),
    }
}

struct PrepareResult {
    canister_id: CanisterId,
    amount: Cycles,
}

fn prepare(args: Args, state: &mut State) -> Result<PrepareResult, Response> {
    let canister_id: CanisterId = state.env.caller();
    let max_amount = state.data.max_top_up_amount;
    let amount = args.amount.map_or(max_amount, |c| min(c, max_amount));

    if let Some(canister) = state.data.canisters.get_mut(&canister_id) {
        if canister.top_up_in_progress() {
            Err(TopUpInProgress)
        } else if let Some(interval) =
            calc_required_wait_period(canister.latest_top_up(), state.data.min_interval, state.env.now())
        {
            Err(Throttled(interval))
        } else if state.env.cycles_balance() < state.data.min_cycles_balance + amount {
            Err(CyclesBalanceTooLow)
        } else {
            canister.set_top_up_in_progress(true);
            Ok(PrepareResult { canister_id, amount })
        }
    } else {
        info!(%canister_id, "Unauthorized canister tried to request cycles");
        Err(NotAuthorized)
    }
}

fn commit(canister_id: &CanisterId, top_up_amount: Option<Cycles>, state: &mut State) {
    if let Some(canister) = state.data.canisters.get_mut(canister_id) {
        canister.set_top_up_in_progress(false);
        if let Some(amount) = top_up_amount {
            canister.record_top_up(amount, state.env.now())
        }
    }
}

fn calc_required_wait_period(
    latest_top_up: Option<TimestampMillis>,
    min_interval: Milliseconds,
    now: TimestampMillis,
) -> Option<Milliseconds> {
    latest_top_up.and_then(|t| (t + min_interval).checked_sub(now))
}
