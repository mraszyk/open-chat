use crate::{RuntimeState, bots::extract_access_context, mutate_state};
use canister_api_macros::update;
use local_user_index_canister::bot_send_message::*;
use rand::Rng;
use types::{
    BotActionScope, BotInitiator, BotMessageContent, ChannelId, Chat, ChatId, CommunityId, MessageId, MessageIndex, UserId,
};

#[update(candid = true, json = true, msgpack = true)]
async fn bot_send_message(args: Args) -> Response {
    let context = match mutate_state(|state| extract_message_access_context(&args, state)) {
        Ok(context) => context,
        Err(response) => return response,
    };

    match context.chat {
        Chat::Direct(chat_id) => {
            send_message_to_user(
                context.bot_id,
                context.bot_name,
                context.initiator,
                chat_id,
                context.thread,
                context.message_id,
                context.user_message_id,
                args.content,
                args.block_level_markdown,
                args.finalised,
            )
            .await
        }
        Chat::Group(chat_id) => {
            send_message_to_group(
                context.bot_id,
                context.bot_name,
                context.initiator,
                chat_id,
                context.thread,
                context.message_id,
                args.content,
                args.block_level_markdown,
                args.finalised,
            )
            .await
        }
        Chat::Channel(community_id, channel_id) => {
            send_message_to_channel(
                context.bot_id,
                context.bot_name,
                context.initiator,
                community_id,
                channel_id,
                context.thread,
                context.message_id,
                args.content,
                args.block_level_markdown,
                args.finalised,
            )
            .await
        }
    }
}

struct MessageAccessContext {
    bot_id: UserId,
    bot_name: String,
    initiator: BotInitiator,
    chat: Chat,
    thread: Option<MessageIndex>,
    message_id: MessageId,
    user_message_id: Option<MessageId>,
}

fn extract_message_access_context(args: &Args, state: &mut RuntimeState) -> Result<MessageAccessContext, Response> {
    use Response::*;

    let context = extract_access_context(&args.auth_token, state).map_err(FailedAuthentication)?;

    let (chat, thread, message_id, user_message_id) = match context.scope {
        BotActionScope::Chat(details) => {
            if let Some(message_id) = args.message_id {
                if matches!(context.initiator, BotInitiator::Command(_)) && message_id != details.message_id {
                    return Err(InvalidRequest(
                        "Message id is already specified in the command access token".to_string(),
                    ));
                }
            }
            let message_id = args.message_id.unwrap_or(details.message_id);
            (details.chat, details.thread, message_id, details.user_message_id)
        }
        BotActionScope::Community(details) => {
            let Some(channel_id) = args.channel_id else {
                return Err(InvalidRequest("Channel must be specified for community scope".to_string()));
            };
            let chat = Chat::Channel(details.community_id, channel_id);
            let message_id = args.message_id.unwrap_or_else(|| state.env.rng().r#gen::<u64>().into());
            (chat, None, message_id, None)
        }
    };

    Ok(MessageAccessContext {
        bot_id: context.bot_id,
        bot_name: context.bot_name,
        initiator: context.initiator,
        chat,
        thread,
        message_id,
        user_message_id,
    })
}

#[allow(clippy::too_many_arguments)]
async fn send_message_to_channel(
    bot_id: UserId,
    bot_name: String,
    initiator: BotInitiator,
    community_id: CommunityId,
    channel_id: ChannelId,
    thread_root_message_index: Option<MessageIndex>,
    message_id: MessageId,
    content: BotMessageContent,
    block_level_markdown: bool,
    finalised: bool,
) -> Response {
    use Response::*;

    match community_canister_c2c_client::c2c_bot_send_message(
        community_id.into(),
        &community_canister::c2c_bot_send_message::Args {
            bot_id,
            initiator,
            channel_id,
            thread_root_message_index,
            message_id,
            content,
            bot_name,
            block_level_markdown,
            finalised,
        },
    )
    .await
    {
        Ok(response) => match response {
            community_canister::c2c_bot_send_message::Response::Success(result) => Success(SuccessResult {
                message_id,
                event_index: result.event_index,
                message_index: result.message_index,
                timestamp: result.timestamp,
                expires_at: result.expires_at,
            }),
            community_canister::c2c_bot_send_message::Response::Error(error) => Error(error),
        },
        Err(error) => Error(error.into()),
    }
}

#[allow(clippy::too_many_arguments)]
async fn send_message_to_group(
    bot_id: UserId,
    bot_name: String,
    initiator: BotInitiator,
    chat_id: ChatId,
    thread_root_message_index: Option<MessageIndex>,
    message_id: MessageId,
    content: BotMessageContent,
    block_level_markdown: bool,
    finalised: bool,
) -> Response {
    use Response::*;

    match group_canister_c2c_client::c2c_bot_send_message(
        chat_id.into(),
        &group_canister::c2c_bot_send_message::Args {
            bot_id,
            initiator,
            thread_root_message_index,
            message_id,
            content,
            bot_name,
            block_level_markdown,
            finalised,
        },
    )
    .await
    {
        Ok(response) => match response {
            group_canister::c2c_bot_send_message::Response::Success(result) => Success(SuccessResult {
                message_id,
                event_index: result.event_index,
                message_index: result.message_index,
                timestamp: result.timestamp,
                expires_at: result.expires_at,
            }),
            group_canister::c2c_bot_send_message::Response::Error(error) => Error(error),
        },
        Err(error) => C2CError(error.reject_code() as i32, error.message().to_string()),
    }
}

#[allow(clippy::too_many_arguments)]
async fn send_message_to_user(
    bot_id: UserId,
    bot_name: String,
    initiator: BotInitiator,
    chat_id: ChatId,
    thread_root_message_index: Option<MessageIndex>,
    message_id: MessageId,
    user_message_id: Option<MessageId>,
    content: BotMessageContent,
    block_level_markdown: bool,
    finalised: bool,
) -> Response {
    use Response::*;

    match user_canister_c2c_client::c2c_bot_send_message(
        chat_id.into(),
        &user_canister::c2c_bot_send_message::Args {
            bot_id,
            initiator,
            thread_root_message_index,
            message_id,
            user_message_id,
            content,
            bot_name,
            block_level_markdown,
            finalised,
        },
    )
    .await
    {
        Ok(response) => match response {
            user_canister::c2c_bot_send_message::Response::Success(result) => Success(SuccessResult {
                message_id,
                event_index: result.event_index,
                message_index: result.message_index,
                timestamp: result.timestamp,
                expires_at: result.expires_at,
            }),
            user_canister::c2c_bot_send_message::Response::Error(error) => Error(error),
        },
        Err(error) => C2CError(error.reject_code() as i32, error.message().to_string()),
    }
}
