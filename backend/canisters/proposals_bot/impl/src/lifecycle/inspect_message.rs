use crate::{RuntimeState, read_state};
use ic_cdk::inspect_message;

#[inspect_message]
fn inspect_message() {
    read_state(accept_if_valid);
}

fn accept_if_valid(state: &RuntimeState) {
    let method_name = ic_cdk::api::msg_method_name().trim_end_matches("_msgpack").to_string();

    let is_valid = match method_name.as_str() {
        "add_governance_canister"
        | "remove_governance_canister"
        | "appoint_admins"
        | "import_proposals_group_into_community" => state.is_caller_governance_principal(),
        "stake_neuron_for_submitting_proposals" | "submit_proposal" | "top_up_neuron" => true,
        _ => false,
    };

    if is_valid {
        ic_cdk::api::accept_message();
    }
}
