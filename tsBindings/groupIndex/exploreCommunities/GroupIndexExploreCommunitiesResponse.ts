// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { GroupIndexExploreCommunitiesSuccessResult } from "./GroupIndexExploreCommunitiesSuccessResult";

export type GroupIndexExploreCommunitiesResponse = { "Success": GroupIndexExploreCommunitiesSuccessResult } | { "TermTooShort": number } | { "TermTooLong": number } | "InvalidTerm" | "InvalidFlags";