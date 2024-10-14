// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ChatId } from "../../shared/ChatId";
import type { CommunityId } from "../../shared/CommunityId";
import type { DeletedCommunityInfo } from "../../shared/DeletedCommunityInfo";
import type { DeletedGroupInfo } from "../../shared/DeletedGroupInfo";

export type GroupIndexActiveGroupsSuccessResult = { timestamp: bigint, active_groups: Array<ChatId>, active_communities: Array<CommunityId>, deleted_groups: Array<DeletedGroupInfo>, deleted_communities: Array<DeletedCommunityInfo>, };
