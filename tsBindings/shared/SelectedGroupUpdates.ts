// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { EventIndex } from "./EventIndex";
import type { GroupMember } from "./GroupMember";
import type { InstalledBotDetails } from "./InstalledBotDetails";
import type { MessageIndex } from "./MessageIndex";
import type { PublicApiKeyDetails } from "./PublicApiKeyDetails";
import type { UserId } from "./UserId";
import type { VersionedRules } from "./VersionedRules";

export type SelectedGroupUpdates = { timestamp: bigint, last_updated: bigint, latest_event_index: EventIndex, members_added_or_updated: Array<GroupMember>, members_removed: Array<UserId>, bots_added_or_updated: Array<InstalledBotDetails>, bots_removed: Array<UserId>, api_keys_generated: Array<PublicApiKeyDetails>, blocked_users_added: Array<UserId>, blocked_users_removed: Array<UserId>, invited_users?: Array<UserId>, pinned_messages_added: Array<MessageIndex>, pinned_messages_removed: Array<MessageIndex>, chat_rules?: VersionedRules, };
