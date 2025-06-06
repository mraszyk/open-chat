use candid::CandidType;
use serde::{Deserialize, Serialize};
use ts_export::ts_export;
use types::{MessageId, Milliseconds, UnitResult, UserId, VideoCallType};

#[ts_export(user, start_video_call)]
#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct Args {
    pub message_id: MessageId,
    pub initiator: UserId,
    pub initiator_username: String,
    pub initiator_display_name: Option<String>,
    pub initiator_avatar_id: Option<u128>,
    pub max_duration: Option<Milliseconds>,
    pub call_type: VideoCallType,
}

pub type Response = UnitResult;
