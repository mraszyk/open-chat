use crate::guards::caller_is_governance_principal;
use crate::{RuntimeState, mutate_state};
use canister_api_macros::proposal;
use canister_tracing_macros::trace;
use notifications_index_canister::upgrade_notifications_canister_wasm::{Response::*, *};
use std::collections::HashSet;
use tracing::{error, info};
use types::BuildVersion;

#[proposal(guard = "caller_is_governance_principal")]
#[trace]
fn upgrade_notifications_canister_wasm(args: Args) -> Response {
    let response = mutate_state(|state| upgrade_notifications_canister_wasm_impl(args, state));
    if !matches!(response, Success) {
        error!(?response, "Failed to upgrade Notifications canister wasm");
    }
    response
}

fn upgrade_notifications_canister_wasm_impl(args: Args, state: &mut RuntimeState) -> Response {
    let version = args.wasm.version;

    if !state.data.test_mode && version < state.data.notifications_canister_wasm_for_new_canisters.version {
        VersionNotHigher
    } else {
        state.data.canisters_requiring_upgrade.clear();
        state.data.notifications_canister_wasm_for_new_canisters = args.wasm.clone();
        state.data.notifications_canister_wasm_for_upgrades = args.wasm;

        let filter = args.filter.unwrap_or_default();
        let include: HashSet<_> = filter.include.into_iter().collect();
        let include_all = include.is_empty();
        let exclude: HashSet<_> = filter.exclude.into_iter().collect();

        for canister_id in state
            .data
            .notifications_canisters
            .iter()
            .filter(|(_, n)| n.wasm_version() != version)
            .map(|(c, _)| *c)
            .filter(|c| include_all || include.contains(c))
            .filter(|c| !exclude.contains(c))
        {
            state.data.canisters_requiring_upgrade.enqueue(canister_id, false);
        }
        crate::jobs::upgrade_canisters::start_job_if_required(state);

        state.data.canisters_requiring_upgrade.clear_failed(BuildVersion {
            major: version.major,
            minor: version.minor,
            patch: version.patch.saturating_sub(100),
        });

        let canisters_queued_for_upgrade = state.data.canisters_requiring_upgrade.count_pending();
        info!(%version, canisters_queued_for_upgrade, "Notifications canister wasm upgraded");
        Success
    }
}
