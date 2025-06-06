use notifications_index_canister::NotificationsIndexEvent;
use timer_job_queues::{TimerJobItem, grouped_timer_job_batch};
use types::{CanisterId, IdempotentEnvelope};
use utils::canister::should_retry_failed_c2c_call;

grouped_timer_job_batch!(
    NotificationCanistersEventBatch,
    CanisterId,
    IdempotentEnvelope<NotificationsIndexEvent>,
    1000
);

impl TimerJobItem for NotificationCanistersEventBatch {
    async fn process(&self) -> Result<(), bool> {
        let args = notifications_canister::c2c_notifications_index::Args {
            events: self.items.clone(),
        };

        let response = notifications_canister_c2c_client::c2c_notifications_index(self.key, &args).await;

        match response {
            Ok(_) => Ok(()),
            Err(error) => {
                let retry = should_retry_failed_c2c_call(error.reject_code(), error.message());
                Err(retry)
            }
        }
    }
}
