use crate::guards::caller_is_community_canister;
use crate::{RuntimeState, read_state};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use group_index_canister::c2c_start_importing_group_into_community::{Response::*, *};
use types::CommunityId;

#[update(guard = "caller_is_community_canister", msgpack = true)]
#[trace]
async fn c2c_start_importing_group_into_community(args: Args) -> Response {
    let community_id = match read_state(|state| prepare(&args, state)) {
        Ok(c) => c,
        Err(response) => return response,
    };

    match group_canister_c2c_client::c2c_start_import_into_community(
        args.group_id.into(),
        &group_canister::c2c_start_import_into_community::Args {
            community_id,
            user_id: args.user_id,
        },
    )
    .await
    {
        Ok(group_canister::c2c_start_import_into_community::Response::Success(total_bytes)) => Success(total_bytes),
        Ok(group_canister::c2c_start_import_into_community::Response::Error(error)) => Error(error),
        Err(error) => Error(error.into()),
    }
}

fn prepare(args: &Args, state: &RuntimeState) -> Result<CommunityId, Response> {
    let community_id = CommunityId::from(state.env.caller());

    if state.data.public_groups.get(&args.group_id).is_some() || state.data.private_groups.get(&args.group_id).is_some() {
        Ok(community_id)
    } else {
        Err(GroupNotFound)
    }
}
