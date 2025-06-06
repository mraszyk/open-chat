use crate::{RuntimeState, read_state};
use canister_api_macros::query;
use types::UserDetails;
use user_index_canister::c2c_lookup_user::{Response::*, *};

#[query(msgpack = true)]
fn c2c_lookup_user(args: Args) -> Response {
    read_state(|state| c2c_lookup_user_impl(args, state))
}

fn c2c_lookup_user_impl(args: Args, state: &RuntimeState) -> Response {
    if let Some(user) = state.data.users.get(&args.user_id_or_principal) {
        let now = state.env.now();
        let is_platform_moderator = state.data.platform_moderators.contains(&user.user_id);
        let is_platform_operator = state.data.platform_operators.contains(&user.user_id);
        let is_diamond_member = user.diamond_membership_details.is_active(now);

        Success(UserDetails {
            principal: user.principal,
            user_id: user.user_id,
            username: user.username.clone(),
            is_bot: user.user_type.is_bot(),
            is_platform_moderator,
            is_platform_operator,
            is_diamond_member,
        })
    } else {
        UserNotFound
    }
}
