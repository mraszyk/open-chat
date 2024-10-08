use crate::{CanisterId, UserId};
use candid::{CandidType, Principal};
use std::fmt::{Debug, Display, Formatter};
use ts_export::ts_export;

#[ts_export]
#[derive(CandidType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChatId(CanisterId);

impl From<Principal> for ChatId {
    fn from(principal: Principal) -> Self {
        ChatId(principal)
    }
}

impl From<ChatId> for CanisterId {
    fn from(chat_id: ChatId) -> Self {
        chat_id.0
    }
}

impl From<UserId> for ChatId {
    fn from(user_id: UserId) -> Self {
        Principal::from(user_id).into()
    }
}

impl Debug for ChatId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Display for ChatId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl AsRef<[u8]> for ChatId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
