// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { PrincipalTS } from "../../shared/PrincipalTS";

export type StorageBucketUploadChunkArgs = { file_id: bigint, hash: [number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number], mime_type: string, accessors: Array<PrincipalTS>, chunk_index: number, chunk_size: number, total_size: bigint, bytes: Array<number>, expiry?: bigint, };
