use ic_cdk::call::RejectCode;

mod accept_cycles;
mod can_spend_cycles;
mod check_cycles_balance;
mod cycles_dispenser_client;

pub use self::cycles_dispenser_client::init_cycles_dispenser_client;
pub use accept_cycles::accept_cycles;
pub use can_spend_cycles::can_spend_cycles;
pub use check_cycles_balance::{MIN_CYCLES_BALANCE, check_cycles_balance, send_low_balance_notification};

pub fn is_out_of_cycles_error(code: RejectCode, message: &str) -> bool {
    code == RejectCode::SysTransient && message.starts_with("IC0207:")
}
