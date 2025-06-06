use crate::{RuntimeState, mutate_state};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use group_index_canister::c2c_update_group::{Response::*, *};
use types::ChatId;

#[update(msgpack = true)]
#[trace]
fn c2c_update_group(args: Args) -> Response {
    mutate_state(|state| c2c_update_group_impl(args, state))
}

fn c2c_update_group_impl(args: Args, state: &mut RuntimeState) -> Response {
    let chat_id = ChatId::from(state.env.caller());
    let now = state.env.now();

    if let Some(group) = state.data.public_groups.get(&chat_id) {
        if !group.name().eq_ignore_ascii_case(&args.name) {
            if state.data.public_group_and_community_names.is_name_taken(&args.name, now) {
                return NameTaken;
            }

            state
                .data
                .public_group_and_community_names
                .rename(group.name(), &args.name, chat_id.into());
        }

        state
            .data
            .public_groups
            .update_group(&chat_id, args.name, args.description, args.avatar_id, args.gate_config);
        Success
    } else if let Some(group) = state.data.private_groups.get(&chat_id) {
        if state.data.public_group_and_community_names.is_name_taken(&args.name, now) {
            return NameTaken;
        }

        let date_created = group.created();

        state.data.private_groups.delete(&chat_id);
        state.data.public_group_and_community_names.insert(&args.name, chat_id.into());

        state.data.public_groups.add(
            chat_id,
            args.name,
            args.description,
            None,
            args.avatar_id,
            args.gate_config,
            date_created,
        );
        Success
    } else {
        ChatNotFound
    }
}
