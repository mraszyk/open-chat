use crate::{RuntimeState, mutate_state};
use group_index_canister::ChildCanisterType;
use ic_cdk::management_canister::CanisterInstallMode;
use ic_cdk_timers::TimerId;
use std::cell::Cell;
use std::time::Duration;
use tracing::trace;
use types::{BuildVersion, CanisterId};
use utils::canister::{FailedUpgrade, WasmToInstall, install};

type CanisterToUpgrade = utils::canister::CanisterToInstall<local_group_index_canister::post_upgrade::Args>;

thread_local! {
    static TIMER_ID: Cell<Option<TimerId>> = Cell::default();
}

pub(crate) fn start_job_if_required(state: &RuntimeState) -> bool {
    if TIMER_ID.get().is_none()
        && (state.data.canisters_requiring_upgrade.count_pending() > 0
            || state.data.canisters_requiring_upgrade.count_in_progress() > 0)
    {
        let timer_id = ic_cdk_timers::set_timer_interval(Duration::ZERO, run);
        TIMER_ID.set(Some(timer_id));
        trace!("'upgrade_canisters' job started");
        true
    } else {
        false
    }
}

fn run() {
    match mutate_state(try_get_next) {
        GetNextResult::Success(canister_to_upgrade) => ic_cdk::futures::spawn(perform_upgrade(canister_to_upgrade)),
        GetNextResult::Continue => {}
        GetNextResult::QueueEmpty => {
            if let Some(timer_id) = TIMER_ID.take() {
                ic_cdk_timers::clear_timer(timer_id);
                trace!("'upgrade_canisters' job stopped");
            }
        }
    }
}

enum GetNextResult {
    Success(CanisterToUpgrade),
    Continue,
    QueueEmpty,
}

fn try_get_next(state: &mut RuntimeState) -> GetNextResult {
    if state.data.canisters_requiring_upgrade.count_in_progress() > 0 {
        return GetNextResult::Continue;
    }
    if state.data.canisters_requiring_upgrade.count_pending() == 0 {
        return GetNextResult::QueueEmpty;
    }

    let (canister_id, force) = match state.data.canisters_requiring_upgrade.try_take_next() {
        Some(c) => c,
        None => return GetNextResult::Continue,
    };

    let new_wasm = &state.data.child_canister_wasms.get(ChildCanisterType::LocalGroupIndex).wasm;
    let new_wasm_version = new_wasm.version;
    let current_wasm_version = match state
        .data
        .local_index_map
        .get(&canister_id)
        .map(|c| c.wasm_version())
        .filter(|v| *v != new_wasm_version || force)
    {
        Some(v) => v,
        None => {
            state.data.canisters_requiring_upgrade.mark_skipped(&canister_id);
            return GetNextResult::Continue;
        }
    };

    GetNextResult::Success(CanisterToUpgrade {
        canister_id,
        current_wasm_version,
        new_wasm_version: new_wasm.version,
        new_wasm: WasmToInstall::Default(new_wasm.module.clone()),
        deposit_cycles_if_needed: false,
        args: local_group_index_canister::post_upgrade::Args {
            wasm_version: new_wasm_version,
        },
        mode: CanisterInstallMode::Upgrade(None),
        stop_start_canister: true,
    })
}

async fn perform_upgrade(canister_to_upgrade: CanisterToUpgrade) {
    let canister_id = canister_to_upgrade.canister_id;
    let from_version = canister_to_upgrade.current_wasm_version;
    let to_version = canister_to_upgrade.new_wasm_version;

    match install(canister_to_upgrade).await {
        Ok(_) => {
            mutate_state(|state| on_success(canister_id, to_version, state));
        }
        Err(_) => {
            mutate_state(|state| on_failure(canister_id, from_version, to_version, state));
        }
    }
}

fn on_success(canister_id: CanisterId, to_version: BuildVersion, state: &mut RuntimeState) {
    if let Some(local_group_index) = state.data.local_index_map.get_mut(&canister_id) {
        local_group_index.set_wasm_version(to_version);

        state.data.canisters_requiring_upgrade.mark_success(&canister_id);
    }
}

fn on_failure(canister_id: CanisterId, from_version: BuildVersion, to_version: BuildVersion, state: &mut RuntimeState) {
    state.data.canisters_requiring_upgrade.mark_failure(FailedUpgrade {
        canister_id,
        from_version,
        to_version,
    });
}
