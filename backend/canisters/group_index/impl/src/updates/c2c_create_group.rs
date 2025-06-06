use crate::model::private_groups::PrivateGroupInfo;
use crate::{RuntimeState, mutate_state, read_state};
use candid::Principal;
use canister_api_macros::update;
use canister_tracing_macros::trace;
use group_index_canister::c2c_create_group::{Response::*, *};
use types::{AccessGateConfig, CanisterId, ChatId, Document, GroupSubtype, UserId};

#[update(msgpack = true)]
#[trace]
async fn c2c_create_group(args: Args) -> Response {
    let avatar_id = Document::id(&args.avatar);

    let (user_id, principal) = match validate_caller().await {
        Ok((u, p)) => (u, p),
        Err(response) => return response,
    };

    let PrepareResult {
        local_group_index_canister,
    } = match mutate_state(|state| prepare(&args.name, args.is_public, state)) {
        Ok(ok) => ok,
        Err(response) => return response,
    };

    let c2c_create_group_args = local_group_index_canister::c2c_create_group::Args {
        created_by_user_id: user_id,
        created_by_user_principal: principal,
        is_public: args.is_public,
        name: args.name.clone(),
        description: args.description.clone(),
        rules: args.rules,
        subtype: args.subtype.clone(),
        avatar: args.avatar,
        history_visible_to_new_joiners: args.history_visible_to_new_joiners,
        messages_visible_to_non_members: args.messages_visible_to_non_members,
        permissions_v2: args.permissions_v2,
        events_ttl: args.events_ttl,
        gate_config: args.gate_config.clone(),
    };

    match local_group_index_canister_c2c_client::c2c_create_group(local_group_index_canister, &c2c_create_group_args).await {
        Ok(local_group_index_canister::c2c_create_group::Response::Success(result)) => {
            mutate_state(|state| {
                commit(
                    CommitArgs {
                        is_public: args.is_public,
                        chat_id: result.chat_id,
                        name: args.name,
                        description: args.description,
                        subtype: args.subtype,
                        avatar_id,
                        gate_config: args.gate_config,
                        local_group_index_canister,
                    },
                    state,
                )
            });
            Success(SuccessResult {
                chat_id: result.chat_id,
                local_user_index_canister_id: result.local_user_index_canister_id,
            })
        }
        Ok(local_group_index_canister::c2c_create_group::Response::Error(error)) => Error(error),
        Ok(local_group_index_canister::c2c_create_group::Response::CyclesBalanceTooLow) => CyclesBalanceTooLow,
        Ok(local_group_index_canister::c2c_create_group::Response::InternalError(_)) => InternalError,
        Err(_) => {
            if args.is_public {
                mutate_state(|state| state.data.public_group_and_community_names.unreserve_name(&args.name));
            }
            InternalError
        }
    }
}

async fn validate_caller() -> Result<(UserId, Principal), Response> {
    let (caller, user_index_canister_id): (UserId, CanisterId) =
        read_state(|state| (state.env.caller().into(), state.data.user_index_canister_id));

    match user_index_canister_c2c_client::c2c_lookup_user(
        user_index_canister_id,
        &user_index_canister::c2c_lookup_user::Args {
            user_id_or_principal: caller.into(),
        },
    )
    .await
    {
        Ok(user_index_canister::c2c_lookup_user::Response::Success(r)) => Ok((caller, r.principal)),
        Ok(user_index_canister::c2c_lookup_user::Response::UserNotFound) => Err(UserNotFound),
        Ok(user_index_canister::c2c_lookup_user::Response::Error(error)) => Err(Error(error)),
        Err(_) => Err(InternalError),
    }
}

struct PrepareResult {
    pub local_group_index_canister: CanisterId,
}

fn prepare(name: &str, is_public: bool, state: &mut RuntimeState) -> Result<PrepareResult, Response> {
    let now = state.env.now();

    if is_public && !state.data.public_group_and_community_names.reserve_name(name, now) {
        return Err(NameTaken);
    }

    if let Some(local_group_index_canister) = state.data.local_index_map.index_for_new_group() {
        Ok(PrepareResult {
            local_group_index_canister,
        })
    } else {
        Err(InternalError)
    }
}

struct CommitArgs {
    is_public: bool,
    chat_id: ChatId,
    name: String,
    description: String,
    subtype: Option<GroupSubtype>,
    avatar_id: Option<u128>,
    gate_config: Option<AccessGateConfig>,
    local_group_index_canister: CanisterId,
}

fn commit(args: CommitArgs, state: &mut RuntimeState) {
    let now = state.env.now();
    if args.is_public {
        state
            .data
            .public_group_and_community_names
            .insert(&args.name, args.chat_id.into());

        state.data.public_groups.add(
            args.chat_id,
            args.name,
            args.description,
            args.subtype,
            args.avatar_id,
            args.gate_config,
            now,
        );
    } else {
        state.data.private_groups.add(PrivateGroupInfo::new(args.chat_id, now));
    }
    state
        .data
        .local_index_map
        .add_group(args.local_group_index_canister, args.chat_id);
}
