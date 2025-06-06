use crate::CanisterId;
use candid::{CandidType, Principal};
use icrc_ledger_types::icrc1::account::Account;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use ts_export::ts_export;

#[ts_export]
#[derive(CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(CanisterId);

impl UserId {
    pub const fn new(canister_id: CanisterId) -> UserId {
        UserId(canister_id)
    }
}

impl From<Principal> for UserId {
    fn from(principal: Principal) -> Self {
        UserId(principal)
    }
}

impl From<UserId> for CanisterId {
    fn from(user_id: UserId) -> Self {
        user_id.0
    }
}

impl From<UserId> for Account {
    fn from(value: UserId) -> Self {
        Account {
            owner: Principal::from(value),
            subaccount: None,
        }
    }
}

impl Debug for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Deref for UserId {
    type Target = Principal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[ts_export]
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub user_id: UserId,
    pub username: String,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct UserDetails {
    pub principal: Principal,
    pub user_id: UserId,
    pub username: String,
    pub is_bot: bool,
    pub is_platform_moderator: bool,
    pub is_platform_operator: bool,
    pub is_diamond_member: bool,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UserType {
    #[default]
    User,
    BotV2,
    Bot,
    OcControlledBot,
    Webhook,
}

impl UserType {
    pub fn is_bot(&self) -> bool {
        matches!(self, UserType::BotV2 | UserType::Bot | UserType::OcControlledBot)
    }

    pub fn is_oc_controlled_bot(&self) -> bool {
        matches!(self, UserType::OcControlledBot)
    }

    pub fn is_3rd_party_bot(&self) -> bool {
        matches!(self, UserType::BotV2 | UserType::Bot)
    }
}
