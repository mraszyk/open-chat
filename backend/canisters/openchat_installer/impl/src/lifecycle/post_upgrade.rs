use crate::lifecycle::{init_env, init_state};
use crate::memory::get_upgrades_memory;
use crate::{Data, read_state};
use canister_logger::LogEntry;
use canister_tracing_macros::trace;
use ic_cdk::management_canister::{CanisterSettings, LogVisibility, UpdateSettingsArgs};
use ic_cdk::post_upgrade;
use openchat_installer_canister::post_upgrade::Args;
use stable_memory::get_reader;
use std::time::Duration;
use tracing::info;
use utils::cycles::init_cycles_dispenser_client;

#[post_upgrade]
#[trace]
fn post_upgrade(args: Args) {
    let memory = get_upgrades_memory();
    let reader = get_reader(&memory);

    let (data, errors, logs, traces): (Data, Vec<LogEntry>, Vec<LogEntry>, Vec<LogEntry>) =
        msgpack::deserialize(reader).unwrap();

    canister_logger::init_with_logs(data.test_mode, errors, logs, traces);

    let env = init_env(data.rng_seed);
    init_cycles_dispenser_client(data.cycles_dispenser_canister_id, data.test_mode);
    init_state(env, data, args.wasm_version);

    let total_instructions = ic_cdk::api::call_context_instruction_counter();
    info!(version = %args.wasm_version, total_instructions, "Post-upgrade complete");

    ic_cdk_timers::set_timer(Duration::ZERO, || ic_cdk::futures::spawn(make_logs_public()));
}

async fn make_logs_public() {
    let canister_ids = read_state(|state| {
        [
            state.data.user_index_canister_id,
            state.data.group_index_canister_id,
            state.data.notifications_index_canister_id,
        ]
    });

    for canister_id in canister_ids {
        ic_cdk::management_canister::update_settings(&UpdateSettingsArgs {
            canister_id,
            settings: CanisterSettings {
                log_visibility: Some(LogVisibility::Public),
                ..Default::default()
            },
        })
        .await
        .unwrap();

        info!(%canister_id, "Updated log visibility");
    }
}
