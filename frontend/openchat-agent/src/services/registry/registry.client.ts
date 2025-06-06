import type { HttpAgent, Identity } from "@dfinity/agent";
import type { DexId, RegistryUpdatesResponse } from "openchat-shared";
import { MsgpackCanisterAgent } from "../canisterAgent/msgpack";
import { updatesResponse } from "./mappers";
import { principalStringToBytes } from "../../utils/mapping";
import {
    RegistryAddMessageFilterArgs,
    RegistryAddMessageFilterResponse,
    RegistryAddRemoveSwapProviderArgs,
    RegistryAddRemoveSwapProviderResponse,
    RegistryRemoveMessageFilterArgs,
    RegistrySetAirdropConfigArgs,
    RegistrySetAirdropConfigResponse,
    RegistrySetTokenEnabledArgs,
    RegistrySetTokenEnabledResponse,
    RegistryUpdatesArgs,
    RegistryUpdatesResponse as TRegistryUpdatesResponse,
} from "../../typebox";
import { apiDexId } from "../common/chatMappersV2";

export class RegistryClient extends MsgpackCanisterAgent {
    private readonly blobUrlPattern: string;

    constructor(identity: Identity, agent: HttpAgent, canisterId: string, blobUrlPattern: string) {
        super(identity, agent, canisterId, "Registry");

        this.blobUrlPattern = blobUrlPattern;
    }

    updates(since?: bigint): Promise<RegistryUpdatesResponse> {
        const args = {
            since,
        };
        return this.executeMsgpackQuery(
            "updates",
            args,
            (resp) => updatesResponse(resp, this.blobUrlPattern, this.canisterId),
            RegistryUpdatesArgs,
            TRegistryUpdatesResponse,
        );
    }

    addRemoveSwapProvider(swapProvider: DexId, add: boolean): Promise<boolean> {
        return this.executeMsgpackUpdate(
            "add_remove_swap_provider",
            {
                swap_provider: apiDexId(swapProvider),
                add,
            },
            (resp) => resp === "Success",
            RegistryAddRemoveSwapProviderArgs,
            RegistryAddRemoveSwapProviderResponse,
        );
    }

    addMessageFilter(regex: string): Promise<boolean> {
        return this.executeMsgpackUpdate(
            "add_message_filter",
            { regex },
            (resp) => {
                if (typeof resp !== "string" && "Success" in resp) {
                    console.log(`New message filter id: ${resp.Success}`);
                    return true;
                } else {
                    console.debug("Error calling add_message_filter", resp);
                    return false;
                }
            },
            RegistryAddMessageFilterArgs,
            RegistryAddMessageFilterResponse,
        );
    }

    removeMessageFilter(id: bigint): Promise<boolean> {
        return this.executeMsgpackUpdate(
            "remove_message_filter",
            { id },
            (resp) => typeof resp === "object" && "Success" in resp,
            RegistryRemoveMessageFilterArgs,
            RegistryAddMessageFilterResponse,
        );
    }

    setAirdropConfig(channelId: number, channelName: string): Promise<boolean> {
        return this.executeMsgpackUpdate(
            "set_airdrop_config",
            {
                channel_id: BigInt(channelId),
                channel_name: channelName,
            },
            (resp) => resp === "Success",
            RegistrySetAirdropConfigArgs,
            RegistrySetAirdropConfigResponse,
        )
    }

    setTokenEnabled(ledger: string, enabled: boolean): Promise<boolean> {
        return this.executeMsgpackUpdate(
            "set_token_enabled",
            {
                ledger_canister_id: principalStringToBytes(ledger),
                enabled,
            },
            (resp) => resp === "Success",
            RegistrySetTokenEnabledArgs,
            RegistrySetTokenEnabledResponse,
        );
    }
}
