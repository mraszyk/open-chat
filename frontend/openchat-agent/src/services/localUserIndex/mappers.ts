import { Principal } from "@dfinity/principal";
import type {
    AccessTokenType,
    BotActionScope,
    ChatEventsArgs,
    ChatEventsBatchResponse,
    ChatEventsResponse,
    GroupAndCommunitySummaryUpdatesResponse,
    JoinCommunityResponse,
    JoinGroupResponse,
    MessageContext,
    RegisterUserResponse,
    CommandArg,
    VerifiedCredentialArgs,
    VideoCallType,
} from "openchat-shared";
import type {
    LocalUserIndexChatEventsEventsSelectionCriteria,
    LocalUserIndexChatEventsEventsArgs,
    LocalUserIndexChatEventsEventsContext,
    LocalUserIndexChatEventsResponse,
    LocalUserIndexGroupAndCommunitySummaryUpdatesResponse,
    LocalUserIndexInviteUsersToChannelResponse,
    LocalUserIndexInviteUsersToCommunityResponse,
    LocalUserIndexInviteUsersToGroupResponse,
    LocalUserIndexJoinChannelResponse,
    LocalUserIndexJoinCommunityResponse,
    LocalUserIndexRegisterUserResponse,
    LocalUserIndexWithdrawFromIcpswapResponse,
    VerifiedCredentialGateArgs as TVerifiedCredentialGateArgs,
    VideoCallType as TVideoCallType,
    BotCommandArg,
    BotCommandArgValue,
    LocalUserIndexAccessTokenV2Response,
    LocalUserIndexAccessTokenV2Args,
    BotActionScope as ApiBotActionScope,
} from "../../typebox";
import {
    toBigInt32,
    CommonResponses,
    MAX_EVENTS,
    MAX_MESSAGES,
    UnsupportedValueError,
    isSuccessfulEventsResponse,
} from "openchat-shared";
import {
    bytesToHexString,
    principalBytesToString,
    principalStringToBytes,
} from "../../utils/mapping";
import {
    apiChatIdentifier,
    communityChannelSummary,
    communitySummary,
    gateCheckFailedReason,
    getEventsSuccess,
    ocError,
} from "../common/chatMappersV2";
import { groupChatSummary, groupChatSummaryUpdates } from "../group/mappersV2";
import { communitySummaryUpdates } from "../community/mappersV2";

export function apiAccessTokenType(domain: AccessTokenType): LocalUserIndexAccessTokenV2Args {
    switch (domain.kind) {
        case "join_video_call":
            return { JoinVideoCall: { chat: apiChatIdentifier(domain.chatId) } };
        case "start_video_call":
            return {
                StartVideoCall: {
                    call_type: apiCallType(domain.callType),
                    chat: apiChatIdentifier(domain.chatId),
                },
            };

        case "bot_action_by_command":
            return {
                BotActionByCommand: {
                    bot_id: principalStringToBytes(domain.botId),
                    scope: apiBotActionScope(domain.scope),
                    command: {
                        name: domain.command.commandName,
                        args: domain.command.arguments
                            .filter(commandArgumentHasValue)
                            .map(apiBotCommandArg),
                        meta: domain.command.meta,
                    },
                },
            };
    }
}

function commandArgumentHasValue(arg: CommandArg): boolean {
    switch (arg.kind) {
        case "user":
            return arg.userId != null;
        default:
            return arg.value != null;
    }
}

export function apiBotActionScope(domain: BotActionScope): ApiBotActionScope {
    switch (domain.kind) {
        case "chat_scope":
            return {
                Chat: {
                    chat: apiChatIdentifier(domain.chatId),
                    thread: domain.threadRootMessageIndex,
                    message_id: domain.messageId,
                    user_message_id: domain.userMessageId,
                },
            };
        case "community_scope":
            return {
                Community: {
                    community_id: principalStringToBytes(domain.communityId.communityId),
                },
            };
    }
}

export function apiBotCommandArg(domain: CommandArg): BotCommandArg {
    return {
        name: domain.name,
        value: apiBotCommandArgValue(domain),
    };
}

export function apiBotCommandArgValue(domain: CommandArg): BotCommandArgValue {
    switch (domain.kind) {
        case "boolean":
            return {
                Boolean: domain.value!,
            };
        case "integer":
            return {
                Integer: domain.value!,
            };
        case "decimal":
            return {
                Decimal: domain.value!,
            };
        case "string":
            return {
                String: domain.value!,
            };
        case "user":
            return {
                User: principalStringToBytes(domain.userId!),
            };
        case "dateTime":
            return {
                DateTime: domain.value!,
            };
    }
}

export function apiCallType(domain: VideoCallType): TVideoCallType {
    if (domain === "broadcast") return "Broadcast";
    if (domain === "default") return "Default";
    throw new UnsupportedValueError("Unexpected VideoCallType received", domain);
}

export function accessTokenResponse(
    value: LocalUserIndexAccessTokenV2Response,
): string | undefined {
    if (typeof value === "object" && "Success" in value) {
        return value.Success;
    }
    console.warn("Unable to get access token: ", value);
    return undefined;
}

export function groupAndCommunitySummaryUpdates(
    value: LocalUserIndexGroupAndCommunitySummaryUpdatesResponse,
): GroupAndCommunitySummaryUpdatesResponse[] {
    const results: GroupAndCommunitySummaryUpdatesResponse[] = [];
    for (const result of value.Success) {
        if (result === "SuccessNoUpdates") {
            results.push({
                kind: "no_updates",
            });
        } else if (result === "NotFound") {
            results.push({
                kind: "not_found",
            });
        } else if ("SuccessGroup" in result) {
            results.push({
                kind: "group",
                value: groupChatSummary(result.SuccessGroup),
            });
        } else if ("SuccessGroupUpdates" in result) {
            results.push({
                kind: "group_updates",
                value: groupChatSummaryUpdates(result.SuccessGroupUpdates),
            });
        } else if ("SuccessCommunity" in result) {
            results.push({
                kind: "community",
                value: communitySummary(result.SuccessCommunity),
            });
        } else if ("SuccessCommunityUpdates" in result) {
            results.push({
                kind: "community_updates",
                value: communitySummaryUpdates(result.SuccessCommunityUpdates),
            });
        } else if ("InternalError" in result) {
            results.push({
                kind: "error",
                error: result.InternalError,
            });
        } else if ("Error" in result) {
            results.push({
                kind: "error",
                error: JSON.stringify(result.Error),
            });
        } else {
            throw new UnsupportedValueError(
                "Unexpected ApiSummaryUpdatesResponse type received",
                result,
            );
        }
    }

    return results;
}

export function chatEventsArgs(eventArgs: ChatEventsArgs): LocalUserIndexChatEventsEventsArgs {
    return {
        context: eventsContext(eventArgs.context),
        args: eventsArgsInner(eventArgs.args),
        latest_known_update: eventArgs.latestKnownUpdate,
    };
}

function eventsContext(context: MessageContext): LocalUserIndexChatEventsEventsContext {
    switch (context.chatId.kind) {
        case "direct_chat":
            return {
                Direct: principalStringToBytes(context.chatId.userId),
            };
        case "group_chat":
            return {
                Group: [
                    principalStringToBytes(context.chatId.groupId),
                    context.threadRootMessageIndex ?? null,
                ],
            };
        case "channel":
            return {
                Channel: [
                    principalStringToBytes(context.chatId.communityId),
                    toBigInt32(context.chatId.channelId),
                    context.threadRootMessageIndex ?? null,
                ],
            };
    }
}

function eventsArgsInner(
    args: ChatEventsArgs["args"],
): LocalUserIndexChatEventsEventsSelectionCriteria {
    switch (args.kind) {
        case "page":
            return {
                Page: {
                    max_messages: MAX_MESSAGES,
                    max_events: MAX_EVENTS,
                    ascending: args.ascending,
                    start_index: args.startIndex,
                },
            };
        case "by_index":
            return {
                ByIndex: {
                    events: args.events,
                },
            };
        case "window": {
            return {
                Window: {
                    mid_point: args.midPoint,
                    max_messages: MAX_MESSAGES,
                    max_events: MAX_EVENTS,
                },
            };
        }
    }
}

export async function chatEventsBatchResponse(
    principal: Principal,
    requests: ChatEventsArgs[],
    value: LocalUserIndexChatEventsResponse,
): Promise<ChatEventsBatchResponse> {
    const responses = [] as ChatEventsResponse[];
    for (let i = 0; i < requests.length; i++) {
        const response = value.Success.responses[i];
        const args = requests[i];

        if (response === "NotFound") {
            responses.push({
                kind: "not_found",
            });
        } else if ("Success" in response) {
            const result = await getEventsSuccess(
                response.Success,
                principal,
                args.context.chatId,
                true
            );
            responses.push(isSuccessfulEventsResponse(result) ? { kind: "success", result } : result);
        } else if ("ReplicaNotUpToDate" in response) {
            responses.push({
                kind: "replica_not_up_to_date",
                replicaTimestamp: response.ReplicaNotUpToDate,
                clientTimestamp: args.latestKnownUpdate ?? BigInt(-1),
            });
        } else if ("InternalError" in response) {
            responses.push({
                kind: "internal_error",
                error: response.InternalError,
            });
        } else {
            responses.push(ocError(response.Error));
        }
    }
    return {
        responses,
        timestamp: value.Success.timestamp,
    };
}

export function joinChannelResponse(
    value: LocalUserIndexJoinChannelResponse,
    communityId: string,
): JoinGroupResponse {
    if (typeof value === "object") {
        if ("Success" in value) {
            return { kind: "success", group: communityChannelSummary(value.Success, communityId) };
        } else if ("AlreadyInChannel" in value) {
            return {
                kind: "success",
                group: communityChannelSummary(value.AlreadyInChannel, communityId),
            };
        } else if ("SuccessJoinedCommunity" in value) {
            return {
                kind: "success_joined_community",
                community: communitySummary(value.SuccessJoinedCommunity),
            };
        } else if ("GateCheckFailed" in value) {
            return {
                kind: "gate_check_failed",
                reason: gateCheckFailedReason(value.GateCheckFailed),
            };
        } else if ("Error" in value) {
            return ocError(value.Error);
        }
    }

    console.warn("Join group failed with: ", value);
    return CommonResponses.failure();
}

export function registerUserResponse(
    value: LocalUserIndexRegisterUserResponse,
): RegisterUserResponse {
    if (value === "UsernameInvalid") {
        return { kind: "username_invalid" };
    }
    if (value === "AlreadyRegistered") {
        return { kind: "already_registered" };
    }
    if (value === "UserLimitReached") {
        return { kind: "user_limit_reached" };
    }
    if (value === "RegistrationInProgress") {
        return { kind: "registration_in_progress" };
    }
    if (value === "CyclesBalanceTooLow") {
        return { kind: "cycles_balance_too_low" };
    }
    if (value === "ReferralCodeInvalid") {
        return { kind: "referral_code_invalid" };
    }
    if (value === "ReferralCodeAlreadyClaimed") {
        return { kind: "referral_code_already_claimed" };
    }
    if (value === "ReferralCodeExpired") {
        return { kind: "referral_code_expired" };
    }
    if ("Success" in value) {
        return {
            kind: "success",
            userId: principalBytesToString(value.Success.user_id),
            icpAccount: bytesToHexString(value.Success.icp_account),
        };
    }
    if ("UsernameTooShort" in value) {
        return { kind: "username_too_short" };
    }
    if ("UsernameTooLong" in value) {
        return { kind: "username_too_long" };
    }
    if ("NotSupported" in value) {
        return { kind: "not_supported" };
    }
    if ("InternalError" in value) {
        return { kind: "internal_error" };
    }
    if ("PublicKeyInvalid" in value) {
        return { kind: "public_key_invalid" };
    }
    if ("Error" in value) {
        return ocError(value.Error);
    }

    throw new UnsupportedValueError("Unexpected ApiRegisterUserResponse type received", value);
}

export function inviteUsersResponse(
    value:
        | LocalUserIndexInviteUsersToGroupResponse
        | LocalUserIndexInviteUsersToChannelResponse
        | LocalUserIndexInviteUsersToCommunityResponse,
): boolean {
    if (value === "Success") {
        return true;
    } else {
        console.warn("InviteUsersResponse was unsuccessful", value);
        return false;
    }
}

export function joinCommunityResponse(
    value: LocalUserIndexJoinCommunityResponse,
): JoinCommunityResponse {
    if (typeof value === "object") {
        if ("Success" in value) {
            return { kind: "success", community: communitySummary(value.Success) };
        } else if ("AlreadyInCommunity" in value) {
            return { kind: "success", community: communitySummary(value.AlreadyInCommunity) };
        } else if ("GateCheckFailed" in value) {
            return {
                kind: "gate_check_failed",
                reason: gateCheckFailedReason(value.GateCheckFailed),
            };
        }
    }

    console.warn("Join community failed with: ", value);
    return CommonResponses.failure();
}

export function apiVerifiedCredentialArgs(
    domain: VerifiedCredentialArgs,
): TVerifiedCredentialGateArgs {
    return {
        user_ii_principal: principalStringToBytes(domain.userIIPrincipal),
        ii_origin: domain.iiOrigin,
        credential_jwts: domain.credentialJwts,
        credential_jwt: domain.credentialJwts[0],
    };
}

export function withdrawFromIcpSwapResponse(
    value: LocalUserIndexWithdrawFromIcpswapResponse,
): boolean {
    console.log("Withdraw from ICPSwap response", value);
    return value === "Success";
}
