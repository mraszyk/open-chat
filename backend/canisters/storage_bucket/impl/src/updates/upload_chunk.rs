use crate::guards::caller_is_known_user;
use crate::model::files::{PutChunkArgs, PutChunkResult};
use crate::model::index_event_batch::EventToSync;
use crate::model::users::{FileStatusInternal, IndexSyncComplete};
use crate::{RuntimeState, mutate_state};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use storage_bucket_canister::upload_chunk_v2::{Response::*, *};
use types::{FileRemoved, RejectedReason};
use utils::file_id::validate_file_id;

#[update(guard = "caller_is_known_user", candid = true, json = true, msgpack = true)]
#[trace]
fn upload_chunk_v2(args: Args) -> Response {
    mutate_state(|state| upload_chunk_impl(args, state))
}

fn upload_chunk_impl(args: Args, state: &mut RuntimeState) -> Response {
    let user_id = state.env.caller();
    let now = state.env.now();
    let user = state.data.users.get(&user_id).unwrap();
    let file_id = args.file_id;

    if !validate_file_id(file_id, state.env.canister_id()) {
        return InvalidFileId;
    }

    let mut index_sync_complete = IndexSyncComplete::No;
    let mut status = None;
    if let Some(status) = user.file_status(&file_id) {
        match status {
            FileStatusInternal::Complete(_) | FileStatusInternal::Rejected(RejectedReason::HashMismatch) => {
                return FileAlreadyExists;
            }
            FileStatusInternal::Rejected(RejectedReason::AllowanceExceeded) => return AllowanceExceeded,
            FileStatusInternal::Rejected(RejectedReason::UserNotFound) => return UserNotFound,
            FileStatusInternal::Rejected(RejectedReason::FileExpired) => return FileExpired,
            FileStatusInternal::Uploading(c) => index_sync_complete = *c,
        }
    } else if args.expiry.is_some_and(|e| e < now) {
        return FileExpired;
    } else {
        status = Some(FileStatusInternal::Uploading(IndexSyncComplete::No));
    }

    let response = match state.data.files.put_chunk(PutChunkArgs::new(user_id, args, now)) {
        PutChunkResult::Success(r) => {
            if r.file_completed {
                status = Some(FileStatusInternal::Complete(index_sync_complete));
            }
            if let Some(file_added) = r.file_added {
                state.data.push_event_to_index(EventToSync::FileAdded(file_added));
                crate::jobs::remove_expired_files::start_job_if_required(state);
            }
            Success
        }
        PutChunkResult::FileAlreadyExists => FileAlreadyExists,
        PutChunkResult::FileTooBig(_) => FileTooBig,
        PutChunkResult::FileExpired => {
            status = Some(FileStatusInternal::Rejected(RejectedReason::FileExpired));
            FileExpired
        }
        PutChunkResult::ChunkAlreadyExists => ChunkAlreadyExists,
        PutChunkResult::ChunkIndexTooHigh => ChunkIndexTooHigh,
        PutChunkResult::ChunkSizeMismatch(_) => ChunkSizeMismatch,
        PutChunkResult::HashMismatch(hm) => {
            // When there is a hash mismatch, the file has already been removed from the list of
            // pending files, so we now need to update the status and tell the index canister to
            // remove the file reference.
            status = Some(FileStatusInternal::Rejected(RejectedReason::HashMismatch));

            // We only need to remove the file reference from the index canister if this file
            // consists of multiple chunks. If the file is a single chunk then the Success case of
            // this match statement will never have been reached so the file reference will not have
            // been added to the index canister.
            if hm.chunk_count > 1 {
                state.data.push_event_to_index(EventToSync::FileRemoved(FileRemoved {
                    file_id,
                    meta_data: hm.meta_data,
                }));
            }

            HashMismatch
        }
    };

    if let Some(status) = status {
        state.data.users.set_file_status(user_id, user, file_id, status);
    }

    response
}
