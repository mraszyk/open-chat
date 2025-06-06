use crate::guards::caller_is_governance_principal;
use crate::{mutate_state, read_state};
use canister_api_macros::proposal;
use canister_tracing_macros::trace;
use proposals_bot_canister::import_proposals_group_into_community::{Response::*, *};
use types::MultiUserChat;

// dfx --identity openchat canister --network ic call proposals_bot import_proposals_group_into_community '(record { governance_canister_id=principal "rrkah-fqaaa-aaaaa-aaaaq-cai"; community_id=pincipal "cbopz-duaaa-aaaaa-qaaka-cai" })'
#[proposal(guard = "caller_is_governance_principal")]
#[trace]
async fn import_proposals_group_into_community(args: Args) -> Response {
    if let Some(group_id) =
        read_state(|state| state.data.nervous_systems.get_chat_id(&args.governance_canister_id)).and_then(|c| c.group_id())
    {
        match community_canister_c2c_client::c2c_import_proposals_group(
            args.community_id.into(),
            &community_canister::c2c_import_proposals_group::Args { group_id },
        )
        .await
        {
            Ok(community_canister::c2c_import_proposals_group::Response::Success(channel_id)) => {
                mutate_state(|state| {
                    state.data.nervous_systems.update_chat_id(
                        args.governance_canister_id,
                        MultiUserChat::Channel(args.community_id, channel_id),
                    );
                });
                Success
            }
            Ok(community_canister::c2c_import_proposals_group::Response::Error(error)) => Error(error),
            Err(error) => Error(error.into()),
        }
    } else {
        NotFound
    }
}
