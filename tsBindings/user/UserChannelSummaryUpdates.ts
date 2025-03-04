// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ChannelId } from "../shared/ChannelId";
import type { MessageIndex } from "../shared/MessageIndex";

export type UserChannelSummaryUpdates = { channel_id: ChannelId, read_by_me_up_to?: MessageIndex, threads_read: Record<MessageIndex, MessageIndex>, archived?: boolean, date_read_pinned?: bigint, };
