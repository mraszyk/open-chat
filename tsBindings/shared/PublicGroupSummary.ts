// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { AccessGate } from "./AccessGate";
import type { AccessGateConfig } from "./AccessGateConfig";
import type { BuildVersion } from "./BuildVersion";
import type { ChatId } from "./ChatId";
import type { EventIndex } from "./EventIndex";
import type { EventWrapperMessage } from "./EventWrapperMessage";
import type { FrozenGroupInfo } from "./FrozenGroupInfo";
import type { GroupSubtype } from "./GroupSubtype";
import type { MessageIndex } from "./MessageIndex";
import type { TSPrincipal } from "./TSPrincipal";

export type PublicGroupSummary = { chat_id: ChatId, local_user_index_canister_id: TSPrincipal, last_updated: bigint, name: string, description: string, subtype?: GroupSubtype, history_visible_to_new_joiners: boolean, messages_visible_to_non_members: boolean, avatar_id?: bigint, latest_message?: EventWrapperMessage, latest_event_index: EventIndex, latest_message_index?: MessageIndex, participant_count: number, wasm_version: BuildVersion, is_public: boolean, frozen?: FrozenGroupInfo, events_ttl?: bigint, events_ttl_last_updated: bigint, gate?: AccessGate, gate_config?: AccessGateConfig, };
