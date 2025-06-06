use crate::ChatInList;
use candid::CandidType;
use serde::{Deserialize, Serialize};
use ts_export::ts_export;
use types::UnitResult;

#[ts_export(user, pin_chat)]
#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct Args {
    pub chat: ChatInList,
}

pub type Response = UnitResult;
