<script lang="ts">
    import {
        type AccountTransaction,
        type AccountTransactions,
        type NamedAccount,
        type OpenChat,
        type ResourceKey,
        cryptoLookup,
        currentUser,
        nervousSystemLookup,
        toRecord,
        ui,
    } from "openchat-client";
    import { getContext, onMount } from "svelte";
    import { i18nKey } from "../../../i18n/i18n";
    import { toastStore } from "../../../stores/toast";
    import type { RemoteData as RD } from "../../../utils/remoteData";
    import Button from "../../Button.svelte";
    import ButtonGroup from "../../ButtonGroup.svelte";
    import FancyLoader from "../../icons/FancyLoader.svelte";
    import ModalContent from "../../ModalContent.svelte";
    import Translatable from "../../Translatable.svelte";
    import CryptoSelector from "../CryptoSelector.svelte";
    import TransactionEndpoint from "./TransactionEndpoint.svelte";

    const client = getContext<OpenChat>("client");

    interface Props {
        ledger: string;
        urlFormat: string;
        onClose: () => void;
    }

    let { ledger = $bindable(), urlFormat = $bindable(), onClose }: Props = $props();

    type LoadingMore<T> = { kind: "loading_more"; data: T };
    type RemoteData = RD<AccountTransactions, string> | LoadingMore<AccountTransactions>;

    let transactionData = $state<RemoteData>({ kind: "loading" });
    let accounts: NamedAccount[] = $state([]);

    function moreTransactionsAvailable(trans: RemoteData): boolean {
        if (trans.kind !== "success") return false;
        if (trans.data.oldestTransactionId === undefined) return false;
        const lastLoaded = trans.data.transactions[trans.data.transactions.length - 1];
        return lastLoaded.id > trans.data.oldestTransactionId;
    }

    onMount(async () => {
        accounts = await client.loadSavedCryptoAccounts();
        loadTransactions();
    });

    function url(id: bigint): string {
        return urlFormat
            .replace("{block_index}", id.toString())
            .replace("{transaction_index}", id.toString())
            .replace("{transaction_hash}", "");
    }

    function openDashboard(id: bigint) {
        window.open(url(id), "_blank");
    }

    function ledgerSelected(l: string, u: string): void {
        transactionData = { kind: "idle" };
        ledger = l;
        urlFormat = u;
        loadTransactions();
    }

    function translateMemo(trans: AccountTransaction): ResourceKey {
        switch (trans.memo) {
            case "OC_MSG":
                return i18nKey("MESSAGE");
            case "OC_SEND":
                return i18nKey("TRANSFER");
            case "OC_TIP":
                return i18nKey("TIP");
            case "OC_PRZ":
                return i18nKey("PRIZE");
            case "OC_PRZCL":
                return i18nKey("PRIZE CLAIM");
            case "OC_PRZRF":
                return i18nKey("PRIZE REFUND");
            case "OC_INS":
                return i18nKey("STREAK INSURANCE");

            default:
                return i18nKey("cryptoAccount.unknownTransactionType");
        }
    }

    function loadTransactions() {
        const nervousSystem = Object.values($nervousSystemLookup).find(
            (n) => n.ledgerCanisterId === ledger,
        );
        const ledgerIndex = nervousSystem?.indexCanisterId;
        if (ledgerIndex !== undefined) {
            let start = undefined;
            if (transactionData.kind === "success") {
                start =
                    transactionData.data.transactions[transactionData.data.transactions.length - 1]
                        .id - 1n;
                transactionData = { kind: "loading_more", data: transactionData.data };
            } else {
                transactionData = { kind: "loading" };
            }
            client
                .getAccountTransactions(ledgerIndex, start)
                .then((result) => {
                    if (result.kind === "failure") {
                        transactionData = { kind: "idle" };
                        toastStore.showFailureToast(i18nKey("cryptoAccount.transactionError"));
                    } else {
                        // Filter out approvals
                        const transactions = result.transactions.filter(
                            (t) => t.kind !== "approve",
                        );
                        if (transactionData.kind === "loading") {
                            transactionData = {
                                kind: "success",
                                data: { ...result, transactions },
                            };
                        } else if (transactionData.kind === "loading_more") {
                            transactionData = {
                                kind: "success",
                                data: {
                                    oldestTransactionId: result.oldestTransactionId,
                                    transactions: [
                                        ...transactionData.data.transactions,
                                        ...transactions,
                                    ],
                                },
                            };
                        }
                    }
                })
                .catch((err) => {
                    console.warn("Error loading transactions: ", err);
                    transactionData = { kind: "idle" };
                    toastStore.showFailureToast(i18nKey("cryptoAccount.transactionError"));
                });
        } else {
            toastStore.showFailureToast(i18nKey("cryptoAccount.transactionError"));
            transactionData = { kind: "idle" };
            console.warn("Could not find ledger index for ledger", ledger, $nervousSystemLookup);
        }
    }
    let accountLookup = $derived(toRecord(accounts, (a) => a.account));
    let tokenDetails = $derived($cryptoLookup[ledger]);
    let snsLedgers = $derived(
        new Set<string>(
            Object.values($nervousSystemLookup)
                .filter((ns) => !ns.isNns)
                .map((ns) => ns.ledgerCanisterId),
        ),
    );
    let moreAvailable = $derived(moreTransactionsAvailable(transactionData));
    let loading = $derived(
        transactionData.kind === "loading" || transactionData.kind === "loading_more",
    );
</script>

<ModalContent fitToContent={!ui.mobileWidth} closeIcon {onClose}>
    {#snippet header()}
        <div class="header">
            <div class="main-title">
                <div><Translatable resourceKey={i18nKey("cryptoAccount.transactions")} /></div>
                <div>
                    <CryptoSelector
                        filter={(t) => snsLedgers.has(t.ledger)}
                        onSelect={ledgerSelected}
                        {ledger} />
                </div>
            </div>
        </div>
    {/snippet}
    {#snippet body()}
        <div class="table-container">
            <div class="table-scroll">
                <table class="data">
                    <thead>
                        <tr>
                            <th
                                ><Translatable
                                    resourceKey={i18nKey(
                                        "cryptoAccount.transactionHeaders.id",
                                    )} /></th>
                            <th
                                ><Translatable
                                    resourceKey={i18nKey(
                                        "cryptoAccount.transactionHeaders.amount",
                                    )} /></th>
                            <th
                                ><Translatable
                                    resourceKey={i18nKey(
                                        "cryptoAccount.transactionHeaders.type",
                                    )} /></th>
                            <th
                                ><Translatable
                                    resourceKey={i18nKey(
                                        "cryptoAccount.transactionHeaders.timestamp",
                                    )} /></th>
                            <th
                                ><Translatable
                                    resourceKey={i18nKey(
                                        "cryptoAccount.transactionHeaders.from",
                                    )} /></th>
                            <th
                                ><Translatable
                                    resourceKey={i18nKey(
                                        "cryptoAccount.transactionHeaders.to",
                                    )} /></th>
                        </tr>
                    </thead>
                    <tbody>
                        {#if transactionData.kind === "success" || transactionData.kind === "loading_more"}
                            {#each transactionData.data.transactions as transaction (transaction.id)}
                                <tr onclick={() => openDashboard(transaction.id)}>
                                    <td>{transaction.id}</td>
                                    <td
                                        >{client.formatTokens(
                                            transaction.amount,
                                            tokenDetails.decimals,
                                        )}</td>
                                    <td class="truncate"
                                        ><Translatable
                                            resourceKey={translateMemo(transaction)} /></td>
                                    <td>{client.toDatetimeString(transaction.timestamp)}</td>
                                    <td class="truncate">
                                        <TransactionEndpoint
                                            accounts={accountLookup}
                                            address={transaction.from}
                                            currentUser={$currentUser} />
                                    </td>
                                    <td class="truncate">
                                        <TransactionEndpoint
                                            accounts={accountLookup}
                                            address={transaction.to}
                                            currentUser={$currentUser} />
                                    </td>
                                </tr>
                            {/each}
                        {:else if transactionData.kind === "loading"}
                            <div class="loading">
                                <FancyLoader />
                            </div>
                        {/if}
                    </tbody>
                </table>
            </div>
        </div>
    {/snippet}
    {#snippet footer()}
        <div>
            <div class="footer">
                <ButtonGroup>
                    <Button
                        secondary
                        onClick={() => loadTransactions()}
                        disabled={!moreAvailable && !loading}
                        {loading}
                        small={!ui.mobileWidth}
                        tiny={ui.mobileWidth}>
                        <Translatable resourceKey={i18nKey("cryptoAccount.loadMoreTransactions")} />
                    </Button>
                    <Button onClick={onClose} small={!ui.mobileWidth} tiny={ui.mobileWidth}>
                        <Translatable resourceKey={i18nKey("close")} />
                    </Button>
                </ButtonGroup>
            </div>
        </div>
    {/snippet}
</ModalContent>

<style lang="scss">
    .table-container {
        height: 400px;
        overflow: hidden;
        border: 1px solid var(--bd);
    }
    .table-scroll {
        overflow-y: auto;
        height: 100%;
    }
    tbody {
        position: relative;
    }
    thead {
        position: sticky;
        top: 0;
        z-index: 1;
    }
    table {
        width: 100%;
        border-collapse: collapse;
        min-width: 600px; // this will scroll horizontally on mobile
    }
    tr {
        border-bottom: 1px solid var(--bd);
    }
    td,
    th {
        border-right: 1px solid var(--bd);
        &:last-child {
            border-right: none;
        }
    }

    th,
    td {
        padding: 8px;
        text-align: left;
    }
    th {
        background-color: var(--button-bg);
        color: var(--button-txt);
    }
    tr {
        cursor: pointer;
    }
    tr:hover {
        background-color: rgba(255, 255, 255, 0.1);
    }
    td:nth-child(2),
    th:nth-child(2) {
        text-align: right;
    }
    .truncate {
        max-width: 150px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .loading {
        width: 80px;
        height: 80px;
        top: 140px;
        left: 50%;
        transform: translateX(-50%);
        position: absolute;
    }
    .main-title {
        flex: auto;
        display: flex;
        align-items: baseline;
        gap: 10px;
        @include font(bold, normal, fs-120);
    }
</style>
