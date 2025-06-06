use crate::activity_notifications::handle_activity_notification;
use crate::guards::caller_is_video_call_operator;
use crate::timer_job_types::TimerJob;
use crate::{RuntimeState, mutate_state, run_regular_jobs};
use canister_tracing_macros::trace;
use group_canister::end_video_call_v2::*;
use ic_cdk::update;
use types::OCResult;

#[update(guard = "caller_is_video_call_operator")]
#[trace]
fn end_video_call_v2(args: Args) -> Response {
    run_regular_jobs();

    mutate_state(|state| end_video_call_impl(args, state)).into()
}

pub(crate) fn end_video_call_impl(args: Args, state: &mut RuntimeState) -> OCResult {
    state.data.timer_jobs.cancel_job(
        |job| {
            if let TimerJob::MarkVideoCallEnded(vc) = job { vc.0 == args } else { false }
        },
    );

    state.data.chat.events.end_video_call(
        args.message_id.into(),
        state.env.now(),
        Some(&mut state.data.event_store_client),
    )?;

    handle_activity_notification(state);
    Ok(())
}
