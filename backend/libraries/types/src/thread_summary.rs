use crate::{EventIndex, TimestampMillis, UserId};
use candid::CandidType;
use ts_export::ts_export;

#[ts_export]
#[derive(CandidType, Clone, Debug)]
pub struct ThreadSummary {
    pub participant_ids: Vec<UserId>,
    pub followed_by_me: bool,
    pub reply_count: u32,
    pub latest_event_index: EventIndex,
    pub latest_event_timestamp: TimestampMillis,
}
