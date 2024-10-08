use crate::{read_state, RuntimeState};
use ic_cdk::inspect_message;

#[inspect_message]
fn inspect_message() {
    read_state(accept_if_valid);
}

fn accept_if_valid(state: &RuntimeState) {
    let method_name = ic_cdk::api::call::method_name().trim_end_matches("_msgpack").to_string();

    // 'inspect_message' only applies to ingress messages so calls to c2c methods should be rejected
    let is_c2c_method = method_name.starts_with("c2c") || method_name == "wallet_receive";
    if is_c2c_method {
        return;
    }

    let is_valid = match method_name.as_str() {
        "claim_daily_chit"
        | "create_canister"
        | "delete_user"
        | "mark_as_online"
        | "mark_suspected_bot"
        | "pay_for_diamond_membership"
        | "set_display_name"
        | "set_moderation_flags"
        | "set_username"
        | "submit_proof_of_unique_personhood"
        | "update_diamond_membership_subscription" => {
            let caller = state.env.caller();
            let is_user = state.data.users.get_by_principal(&caller).is_some();
            is_user
        }
        "add_referral_codes" => state.is_caller_dev_team_dfx_principal(),
        "suspend_user" | "unsuspend_user" => state.is_caller_platform_moderator(),
        "set_user_upgrade_concurrency" | "set_diamond_membership_fees" => state.is_caller_platform_operator(),
        "add_platform_moderator"
        | "add_platform_operator"
        | "remove_platform_moderator"
        | "remove_platform_operator"
        | "assign_platform_moderators_group"
        | "set_max_concurrent_user_canister_upgrades"
        | "add_local_user_index_canister"
        | "upgrade_user_canister_wasm"
        | "upgrade_local_user_index_canister_wasm"
        | "mark_local_user_index_full"
        | "register_external_achievement"
        | "suspected_bots" => state.is_caller_governance_principal(),
        "create_challenge" | "modclub_callback" => true,
        _ => false,
    };

    if is_valid {
        ic_cdk::api::call::accept_message();
    }
}
