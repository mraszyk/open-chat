/* eslint-disable @typescript-eslint/no-explicit-any */
import { IDL } from "@dfinity/candid";
import { Principal } from "@dfinity/principal";
import { type ExternalBot } from "openchat-client";

export const OC_GOVERNANCE_CANISTER_ID = "2jvtu-yqaaa-aaaaq-aaama-cai";

export function createAddTokenPayload(
    ledgerCanisterId: string,
    userId: string,
    infoUrl: string,
    transactionUrlFormat: string,
): Uint8Array {
    return new Uint8Array(
        IDL.encode(
            [
                IDL.Record({
                    info_url: IDL.Text,
                    payer: IDL.Opt(IDL.Principal),
                    token_standard: IDL.Variant({ icrc1: IDL.Null }),
                    ledger_canister_id: IDL.Principal,
                    transaction_url_format: IDL.Text,
                }),
            ],
            [
                {
                    info_url: infoUrl,
                    payer: [Principal.fromText(userId)],
                    token_standard: { icrc1: null },
                    ledger_canister_id: Principal.fromText(ledgerCanisterId),
                    transaction_url_format: transactionUrlFormat,
                },
            ],
        ),
    );
}

export function createUpdateTokenPayload(
    ledgerCanisterId: string,
    name: string | undefined,
    symbol: string | undefined,
    infoUrl: string | undefined,
    transactionUrlFormat: string | undefined,
): Uint8Array {
    return new Uint8Array(
        IDL.encode(
            [
                IDL.Record({
                    info_url: IDL.Opt(IDL.Text),
                    name: IDL.Opt(IDL.Text),
                    ledger_canister_id: IDL.Principal,
                    symbol: IDL.Opt(IDL.Text),
                    transaction_url_format: IDL.Opt(IDL.Text),
                }),
            ],
            [
                {
                    info_url: optionalStringToCandid(infoUrl),
                    name: optionalStringToCandid(name),
                    ledger_canister_id: Principal.fromText(ledgerCanisterId),
                    symbol: optionalStringToCandid(symbol),
                    transaction_url_format: optionalStringToCandid(transactionUrlFormat),
                },
            ],
        ),
    );
}

export function createPublishExternalBotPayload(bot: ExternalBot): Uint8Array {
    return new Uint8Array(
        IDL.encode(
            [
                IDL.Record({
                    bot_id: IDL.Principal,
                }),
            ],
            [
                {
                    bot_id: Principal.fromText(bot.id),
                },
            ],
        ),
    );
}

export function createRegisterExternalAchievementPayload(
    id: number,
    userId: string,
    name: string,
    url: string,
    logo: string,
    canisterId: string,
    chitReward: number,
    expiryTimestampMillis: bigint,
    maxAwards: number,
): Uint8Array {
    return new Uint8Array(
        IDL.encode(
            [
                IDL.Record({
                    id: IDL.Nat32,
                    submitted_by: IDL.Principal,
                    name: IDL.Text,
                    url: IDL.Text,
                    logo: IDL.Text,
                    canister_id: IDL.Principal,
                    chit_reward: IDL.Nat32,
                    expires: IDL.Nat64,
                    max_awards: IDL.Nat32,
                }),
            ],
            [
                {
                    id,
                    submitted_by: Principal.fromText(userId),
                    name,
                    url,
                    logo,
                    canister_id: Principal.fromText(canisterId),
                    chit_reward: chitReward,
                    expires: expiryTimestampMillis,
                    max_awards: maxAwards,
                },
            ],
        ),
    );
}

function optionalStringToCandid(value: string | undefined): [string] | [] {
    return value !== undefined && value.length > 0 ? [value] : [];
}
