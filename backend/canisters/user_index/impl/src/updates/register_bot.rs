use std::collections::HashMap;

use crate::model::user_map::Bot;
use crate::model::{MAX_AVATAR_SIZE, MAX_COMMANDS, MAX_DESCRIPTION_LEN};
use crate::{mutate_state, read_state, RuntimeState};
use candid::Principal;
use canister_api_macros::update;
use canister_tracing_macros::trace;
use constants::{ONE_GB, USER_LIMIT};
use event_store_producer::EventBuilder;
use local_user_index_canister::{BotRegistered, UserIndexEvent};
use rand::RngCore;
use storage_index_canister::add_or_update_users::UserConfig;
use tracing::error;
use types::BotRegistrationStatus;
use types::{UserId, UserType};
use url::Url;
use user_index_canister::register_bot::{Response::*, *};
use utils::document::try_parse_data_url;
use utils::text_validation::{validate_username, UsernameValidationError};

#[update(msgpack = true)]
#[trace]
fn register_bot(args: Args) -> Response {
    if read_state(|state| state.data.test_mode) {
        mutate_state(|state| register_bot_impl(args, state));
    }

    Success
}

fn register_bot_impl(args: Args, state: &mut RuntimeState) {
    let name = &args.name;
    let error_prefix = format!("register bot {name}:");

    if let Err(message) = validate_request(&args, state) {
        error!("{error_prefix} {message}");
        return;
    }

    let avatar = if let Some(data_url) = args.avatar.as_ref() {
        match try_parse_data_url(data_url) {
            Some(a) => Some(a),
            None => {
                error!("{error_prefix} invalid avatar");
                return;
            }
        }
    } else {
        None
    };

    let Some(user_id) = generate_random_user_id(state) else {
        error!("{error_prefix} can't generate unique user id!");
        return;
    };

    let now = state.env.now();

    state.data.users.register(
        args.principal,
        user_id,
        args.name.clone(),
        None,
        now,
        None,
        UserType::BotV2,
        Some(Bot {
            name: args.name.clone(),
            owner: args.owner,
            endpoint: args.endpoint.clone(),
            description: args.definition.description.clone(),
            commands: args.definition.commands.clone(),
            autonomous_config: args.definition.autonomous_config.clone(),
            last_updated: now,
            avatar,
            installations: HashMap::new(),
            registration_status: BotRegistrationStatus::Private(args.permitted_install_location),
        }),
    );

    state.push_event_to_all_local_user_indexes(
        UserIndexEvent::BotRegistered(BotRegistered {
            bot_id: user_id,
            owner_id: args.owner,
            user_principal: args.principal,
            name: args.name.clone(),
            commands: args.definition.commands.clone(),
            autonomous_config: args.definition.autonomous_config.clone(),
            permitted_install_location: args.permitted_install_location,
        }),
        None,
    );

    state.data.storage_index_user_sync_queue.push(
        state.data.storage_index_canister_id,
        UserConfig {
            user_id: args.principal,
            byte_limit: ONE_GB,
        },
    );

    state.data.event_store_client.push(
        EventBuilder::new("user_registered", now)
            .with_user(user_id.to_string(), true)
            .with_source(state.env.canister_id().to_string(), false)
            .with_json_payload(&crate::UserRegisteredEventPayload {
                referred: false,
                is_bot: true,
            })
            .build(),
    );
}

fn validate_request(args: &Args, state: &RuntimeState) -> Result<(), String> {
    let owner_id = &args.owner;

    if args.principal == Principal::anonymous() {
        return Err("principal cannot be anonymous".to_string());
    }

    if state.data.users.get_by_principal(&args.principal).is_some() {
        return Err("already registered".to_string());
    }

    match validate_username(&args.name) {
        Ok(_) => {}
        Err(UsernameValidationError::TooShort(_)) => return Err("name too short".to_string()),
        Err(UsernameValidationError::TooLong(_)) => return Err("name too short".to_string()),
        Err(UsernameValidationError::Invalid) => return Err("name invalid".to_string()),
    };

    if args.avatar.as_ref().is_some_and(|a| a.len() > MAX_AVATAR_SIZE) {
        return Err("avatar too big".to_string());
    }

    if args.definition.description.len() > MAX_DESCRIPTION_LEN {
        return Err("description too long".to_string());
    }

    if args.definition.commands.len() > MAX_COMMANDS {
        return Err("too many commands".to_string());
    }

    if Principal::from_text(&args.endpoint).is_err() && Url::parse(&args.endpoint).is_err() {
        return Err("endpoint invalid".to_string());
    }

    if state.data.users.len() >= USER_LIMIT {
        return Err(format!("user limit reached {USER_LIMIT}"));
    }

    let Some(owner) = state.data.users.get_by_user_id(&args.owner) else {
        return Err(format!("owner not found {owner_id}"));
    };

    if !matches!(owner.user_type, UserType::User) {
        return Err(format!("owner must be a user {owner_id}"));
    }

    if owner.suspension_details.is_some() {
        return Err(format!("owner must not be suspended {owner_id}"));
    }

    if state.data.users.does_username_exist(&args.name, true) {
        return Err("bot name already exists".to_string());
    }

    Ok(())
}

fn generate_random_user_id(state: &mut RuntimeState) -> Option<UserId> {
    let mut user_id_bytes: [u8; 8] = [0; 8];

    for _ in 0..10 {
        state.env.rng().fill_bytes(&mut user_id_bytes);
        let user_id = Principal::from_slice(&user_id_bytes).into();
        if state.data.users.get_by_user_id(&user_id).is_none() {
            return Some(user_id);
        }
    }

    None
}
