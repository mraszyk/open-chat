use crate::guards::caller_is_user_index_canister;
use crate::{Data, RuntimeState, mutate_state, read_state};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use itertools::Itertools;
use local_user_index_canister::ChildCanisterType;
use local_user_index_canister::c2c_upgrade_user_canister_wasm::*;
use sha256::sha256;
use std::cmp::Reverse;
use std::collections::HashSet;
use tracing::info;
use types::{
    BuildVersion, CanisterId, CanisterWasm, ChunkedCanisterWasm, Hash, TimestampMillis, UpgradeChunkedCanisterWasmResponse::*,
    UserId,
};
use utils::canister::{should_perform_upgrade, upload_wasm_in_chunks};

#[update(guard = "caller_is_user_index_canister", msgpack = true)]
#[trace]
async fn c2c_upgrade_user_canister_wasm(args: Args) -> Response {
    let PrepareResult { this_canister_id, wasm } = match read_state(|state| prepare(&args, state)) {
        Ok(ok) => ok,
        Err(response) => return response,
    };

    let active_users_filter = if let Some(active_since) = args.filter.as_ref().and_then(|f| f.active_since) {
        match get_users_active_since(active_since).await {
            Ok(users) => Some(users),
            Err(error) => return InternalError(error),
        }
    } else {
        None
    };

    utils::canister::clear_chunk_store(this_canister_id).await.unwrap();

    let chunks = upload_wasm_in_chunks(&wasm.module, this_canister_id).await.unwrap();

    mutate_state(|state| commit(args, wasm, chunks, active_users_filter, state))
}

struct PrepareResult {
    this_canister_id: CanisterId,
    wasm: CanisterWasm,
}

fn prepare(args: &Args, state: &RuntimeState) -> Result<PrepareResult, Response> {
    let wasm = state.data.child_canister_wasms.wasm_from_chunks(ChildCanisterType::User);
    let wasm_hash = sha256(&wasm);

    if wasm_hash != args.wasm_hash {
        Err(HashMismatch(wasm_hash))
    } else if !state.data.test_mode && Some(args.version) <= min_canister_version(&state.data) {
        Err(VersionNotHigher)
    } else {
        Ok(PrepareResult {
            this_canister_id: state.env.canister_id(),
            wasm: CanisterWasm {
                version: args.version,
                module: wasm,
            },
        })
    }
}

fn commit(
    args: Args,
    wasm: CanisterWasm,
    chunks: Vec<Hash>,
    active_users_filter: Option<HashSet<UserId>>,
    state: &mut RuntimeState,
) -> Response {
    state.data.canisters_requiring_upgrade.clear();
    let version = args.version;
    let wasm_hash = args.wasm_hash;

    state
        .data
        .child_canister_wasms
        .set(ChildCanisterType::User, ChunkedCanisterWasm { wasm, chunks, wasm_hash });

    let filter = args.filter.unwrap_or_default();

    for canister_id in state
        .data
        .local_users
        .iter()
        .filter(|(user_id, _)| active_users_filter.as_ref().is_none_or(|a| a.contains(user_id)))
        .filter(|(user_id, user)| {
            should_perform_upgrade((**user_id).into(), user.wasm_version, version, &filter, state.data.test_mode)
                && !state.data.global_users.is_bot(user_id)
        })
        .map(|(user_id, _)| CanisterId::from(*user_id))
        .sorted_by_key(|&c| Reverse(state.data.global_users.diamond_membership_expiry_date(&c.into())))
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
    info!(%version, canisters_queued_for_upgrade, "User canister wasm upgraded");
    Success
}

fn min_canister_version(data: &Data) -> Option<BuildVersion> {
    data.local_users.iter().map(|(_, u)| u.wasm_version).min()
}

// Compiles the list of users who have been active since the specified timestamp.
// Users are included if either they were created before the cutoff but have been online since it,
// or if they were created after the cutoff.
async fn get_users_active_since(since: TimestampMillis) -> Result<HashSet<UserId>, String> {
    let (online_users_canister_id, users_created_before_cutoff, now) = read_state(|state| {
        (
            state.data.online_users_canister_id,
            state
                .data
                .local_users
                .iter()
                .filter(|(_, u)| u.date_created < since)
                .map(|(u, _)| *u)
                .collect::<Vec<_>>(),
            state.env.now(),
        )
    });

    let max_duration_since_last_online = now.saturating_sub(since);
    let mut active_users = HashSet::new();
    for batch in &users_created_before_cutoff.into_iter().chunks(10_000) {
        let online_users_canister::last_online::Response::Success(users) = online_users_canister_c2c_client::last_online(
            online_users_canister_id,
            &online_users_canister::last_online::Args {
                user_ids: batch.collect(),
            },
        )
        .await
        .map_err(|e| format!("{e:?}"))?;

        active_users.extend(
            users
                .into_iter()
                .filter(|u| u.duration_since_last_online < max_duration_since_last_online)
                .map(|u| u.user_id),
        );
    }

    read_state(|state| {
        active_users.extend(
            state
                .data
                .local_users
                .iter()
                .filter(|(_, u)| u.date_created >= since)
                .map(|(u, _)| *u),
        );
    });
    Ok(active_users)
}
