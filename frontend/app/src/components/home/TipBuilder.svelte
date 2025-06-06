<script lang="ts">
    import type {
        Message,
        MessageContext,
        OpenChat,
        PendingCryptocurrencyTransfer,
    } from "openchat-client";
    import {
        cryptoBalance as cryptoBalanceStore,
        cryptoLookup,
        currentUser,
        exchangeRatesLookupStore as exchangeRatesLookup,
        lastCryptoSent,
        ui,
    } from "openchat-client";
    import { getContext, onMount } from "svelte";
    import { _ } from "svelte-i18n";
    import { quadOut } from "svelte/easing";
    import { Tween } from "svelte/motion";
    import { fade } from "svelte/transition";
    import { i18nKey } from "../../i18n/i18n";
    import { pinNumberErrorMessageStore } from "../../stores/pinNumber";
    import { toastStore } from "../../stores/toast";
    import Button from "../Button.svelte";
    import ButtonGroup from "../ButtonGroup.svelte";
    import ErrorMessage from "../ErrorMessage.svelte";
    import ModalContent from "../ModalContent.svelte";
    import Overlay from "../Overlay.svelte";
    import Translatable from "../Translatable.svelte";
    import AccountInfo from "./AccountInfo.svelte";
    import BalanceWithRefresh from "./BalanceWithRefresh.svelte";
    import CryptoSelector from "./CryptoSelector.svelte";
    import TipButton from "./TipButton.svelte";
    import TokenInput from "./TokenInput.svelte";

    const client = getContext<OpenChat>("client");
    const increments: Increment[] = [1, 10, 100];
    type Increment = 1 | 10 | 100;

    interface Props {
        ledger: string;
        msg: Message;
        messageContext: MessageContext;
        onClose: () => void;
    }

    let { ledger = $bindable(), msg, messageContext, onClose }: Props = $props();

    const tweenOptions = {
        duration: 800,
        easing: quadOut,
    };

    let refreshing = $state(false);
    let error: string | undefined = $state(undefined);
    let toppingUp = $state(false);
    let tokenChanging = $state(true);
    let balanceWithRefresh: BalanceWithRefresh;
    let dollar = $state<HTMLElement | undefined>();
    let dollarTop = new Tween(-1000, tweenOptions);
    let dollarOpacity = new Tween(0, tweenOptions);
    let dollarScale = new Tween(0, tweenOptions);
    let centAmount = $state(0);
    let showCustomTip = $state(false);
    let validAmount: boolean = $state(false);
    let draftAmount = $state(0n);

    onMount(() => {
        let d = document.getElementById("tip-dollar");
        if (!d) {
            d = document.createElement("div");
            d.id = "tip-dollar";
            d.className = "tip-dollar";
            const t = document.createTextNode("🤑");
            d.appendChild(t);
            document.body.appendChild(d);
        }
        dollar = d;
    });

    function amountLabel(n: Increment): string {
        return `$${(n / 100).toFixed(2)}`;
    }

    function calculateCentAmount(e8s: bigint, exchangeRate: number): number {
        const tokens = Number(e8s) / Math.pow(10, tokenDetails.decimals);
        const dollar = tokens * exchangeRate;
        const cents = dollar * 100;
        return Math.round(cents);
    }

    function to2SigFigs(num: number): number {
        return parseFloat(num.toPrecision(2));
    }

    function calculateAmount(centAmount: number, exchangeRate: number): bigint {
        const e8s = ((centAmount / 100) * Math.pow(10, tokenDetails.decimals)) / exchangeRate;
        return BigInt(Math.round(e8s));
    }

    function reset() {
        balanceWithRefresh.refresh();
    }

    function cancel() {
        toppingUp = false;
        onClose();
    }

    function onBalanceRefreshed() {
        onBalanceRefreshFinished();
        error = undefined;
    }

    function onBalanceRefreshError(err: string) {
        onBalanceRefreshFinished();
        error = err;
    }

    function onBalanceRefreshFinished() {
        toppingUp = false;
        tokenChanging = false;
        if (remainingBalance < 0) {
            remainingBalance = 0n;
            draftAmount = cryptoBalance - tokenDetails.transferFee;
            if (draftAmount < 0) {
                draftAmount = 0n;
            }
        }
    }

    function bounceMoneyMouthFrom(target: HTMLElement) {
        if (!dollar) return;
        const buttonRect = target.getBoundingClientRect();
        const hDiff = buttonRect.height - dollar.clientHeight;
        const wDiff = buttonRect.width - dollar.clientWidth;
        const top = buttonRect.top + hDiff / 2;
        const left = buttonRect.left + wDiff / 2;

        Promise.all([
            dollarTop.set(top, { duration: 0 }),
            dollarOpacity.set(1, { duration: 0 }),
            dollarScale.set(0, { duration: 0 }),
        ]).then(() => {
            dollarTop.target = top - 300;
            dollarOpacity.target = 0;
            dollarScale.target = 2;
            dollarOpacity.set(0);
        });
        dollar.style.left = `${left}px`;
    }

    function clickAmount(e: MouseEvent, increment: Increment) {
        e.preventDefault();
        bounceMoneyMouthFrom(e.target as HTMLElement);
        centAmount += increment;
        draftAmount = calculateAmount(centAmount, exchangeRate);
    }

    function maxAmount(balance: bigint): bigint {
        return balance - transferFees;
    }

    function send(e: Event) {
        e.preventDefault();
        const transfer: PendingCryptocurrencyTransfer = {
            kind: "pending",
            ledger,
            token: tokenDetails.symbol,
            recipient: msg.sender,
            amountE8s: draftAmount,
            feeE8s: tokenDetails.transferFee,
            createdAtNanos: BigInt(Date.now()) * 1_000_000n,
        };
        lastCryptoSent.set(ledger);

        const currentTip = (msg.tips[transfer.ledger] ?? {})[$currentUser.userId] ?? 0n;

        client.tipMessage(messageContext, msg.messageId, transfer, currentTip).then((resp) => {
            if (resp.kind === "failure") {
                toastStore.showFailureToast(i18nKey("tip.failure"));
            } else if (resp.kind !== "success") {
                toastStore.showFailureToast(pinNumberErrorMessageStore);
            }
        });

        onClose();
    }
    let tokenDetails = $derived($cryptoLookup[ledger]);
    let cryptoBalance = $derived($cryptoBalanceStore[ledger] ?? 0n);
    let exchangeRate = $derived(
        to2SigFigs($exchangeRatesLookup[tokenDetails.symbol.toLowerCase()]?.toUSD ?? 0),
    );
    $effect(() => {
        if (ledger !== undefined) {
            // reset when ledger changes
            centAmount = 0;
            tokenChanging = true;
            draftAmount = 0n;
        }
    });
    let displayDraftAmount = $derived(client.formatTokens(draftAmount, tokenDetails.decimals));
    let displayFee = $derived(client.formatTokens(tokenDetails.transferFee, tokenDetails.decimals));
    let remainingBalance = $state(0n);
    $effect(() => {
        remainingBalance =
            draftAmount > 0n
                ? cryptoBalance - draftAmount - tokenDetails.transferFee
                : cryptoBalance;
    });
    let valid = $derived(
        draftAmount > 0n && remainingBalance >= 0n && error === undefined && !tokenChanging,
    );
    let zero = $derived(cryptoBalance <= tokenDetails.transferFee && !tokenChanging);
    let transferFees = $derived(tokenDetails.transferFee);
    $effect(() => {
        centAmount = calculateCentAmount(draftAmount, exchangeRate);
    });
    $effect(() => {
        if (dollar) {
            dollar.style.setProperty("top", `${dollarTop.current}px`);
            dollar.style.setProperty("opacity", `${dollarOpacity.current}`);
            dollar.style.setProperty(
                "transform",
                `scale(${dollarScale.current}) rotate(${dollarScale.current}turn)`,
            );
        }
    });
</script>

<Overlay dismissible>
    <ModalContent fill>
        {#snippet header()}
            <span class="header">
                <div class="left">
                    <div class="main-title">
                        <div><Translatable resourceKey={i18nKey("tip.title")} /></div>
                        <div>
                            <CryptoSelector bind:ledger />
                        </div>
                    </div>
                </div>
                <BalanceWithRefresh
                    bind:toppingUp
                    bind:this={balanceWithRefresh}
                    {ledger}
                    value={remainingBalance}
                    label={i18nKey("cryptoAccount.shortBalanceLabel")}
                    bold
                    showTopUp
                    bind:refreshing
                    onRefreshed={onBalanceRefreshed}
                    onError={onBalanceRefreshError} />
            </span>
        {/snippet}
        {#snippet body()}
            <form onsubmit={send}>
                <div class="body" class:zero={zero || toppingUp}>
                    {#if zero || toppingUp}
                        <AccountInfo {ledger} />
                        {#if zero}
                            <p>
                                <Translatable
                                    resourceKey={i18nKey("tokenTransfer.zeroBalance", {
                                        token: tokenDetails.symbol,
                                    })} />
                            </p>
                        {/if}
                        <p><Translatable resourceKey={i18nKey("tokenTransfer.makeDeposit")} /></p>
                    {:else}
                        {#if exchangeRate > 0}
                            <div class="amounts">
                                {#each increments as increment}
                                    <TipButton
                                        label={i18nKey(amountLabel(increment))}
                                        onClick={(e) => clickAmount(e, increment)}
                                        disabled={exchangeRate === 0 ||
                                            calculateAmount(centAmount + increment, exchangeRate) >
                                                cryptoBalance - tokenDetails.transferFee} />
                                {/each}
                            </div>
                            <div in:fade|local={{ duration: 300 }} class="message">
                                {#if draftAmount > 0}
                                    <div class="summary">
                                        <div class="dollar-amount">
                                            ${(centAmount / 100).toFixed(2)}
                                        </div>
                                        <div class="token-amount">
                                            {displayDraftAmount}
                                            {tokenDetails.symbol}
                                        </div>
                                        <div class="fee">
                                            <Translatable
                                                resourceKey={i18nKey("tip.plusFee", {
                                                    fee: displayFee,
                                                    token: tokenDetails.symbol,
                                                })} />
                                        </div>
                                    </div>
                                {:else}
                                    <div class="summary">
                                        <Translatable resourceKey={i18nKey("tip.advice")} />
                                    </div>
                                {/if}
                            </div>
                        {/if}
                        <div class="custom-tip">
                            {#if !showCustomTip && exchangeRate > 0}
                                <!-- svelte-ignore a11y_click_events_have_key_events -->
                                <!-- svelte-ignore a11y_missing_attribute -->
                                <a
                                    role="button"
                                    tabindex="0"
                                    onclick={() => (showCustomTip = true)}
                                    class="options"
                                    in:fade|local={{ duration: 500 }}
                                    class:expanded={showCustomTip}>
                                    <Translatable resourceKey={i18nKey("tip.showCustom")} />
                                </a>
                            {/if}

                            {#if showCustomTip || exchangeRate <= 0}
                                <div in:fade|local={{ duration: 500 }} class="custom-tip-amount">
                                    <TokenInput
                                        {ledger}
                                        {transferFees}
                                        bind:valid={validAmount}
                                        maxAmount={maxAmount(cryptoBalance)}
                                        bind:amount={draftAmount} />
                                </div>
                            {/if}
                        </div>
                        {#if error !== undefined}
                            <div class="error">
                                <ErrorMessage>{$_(error)}</ErrorMessage>
                            </div>
                        {/if}
                    {/if}
                </div>
            </form>
        {/snippet}
        {#snippet footer()}
            <span>
                <ButtonGroup>
                    <Button small={!ui.mobileWidth} tiny={ui.mobileWidth} secondary onClick={cancel}
                        ><Translatable resourceKey={i18nKey("cancel")} /></Button>
                    {#if toppingUp || zero}
                        <Button
                            small={!ui.mobileWidth}
                            disabled={refreshing}
                            loading={refreshing}
                            tiny={ui.mobileWidth}
                            onClick={reset}
                            ><Translatable resourceKey={i18nKey("refresh")} /></Button>
                    {:else}
                        <Button
                            small={!ui.mobileWidth}
                            disabled={!valid}
                            tiny={ui.mobileWidth}
                            onClick={send}
                            ><Translatable resourceKey={i18nKey("tokenTransfer.send")} /></Button>
                    {/if}
                </ButtonGroup>
            </span>
        {/snippet}
    </ModalContent>
</Overlay>

<style lang="scss">
    .header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: $sp2;

        .left {
            flex: auto;
            display: flex;
            align-items: center;
            gap: $sp4;

            .main-title {
                flex: auto;
                display: flex;
                align-items: baseline;
                gap: 10px;
                @include font(bold, normal, fs-120);
            }
        }
    }

    .body {
        transition: background-color 100ms ease-in-out;
        @include font(book, normal, fs-100, 28);
        position: relative;

        &.zero {
            padding: 0 $sp4;
        }
    }

    .message {
        text-align: center;
        margin-bottom: $sp4;
    }

    .error {
        text-align: center;
    }

    .summary {
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--txt);

        .dollar-amount,
        .fee,
        .token-amount {
            padding: 0 $sp4;
        }

        .dollar-amount,
        .token-amount {
            border-right: 1px solid var(--txt-light);
        }

        .fee {
            color: var(--txt-light);
            @include font(light, normal, fs-70);
        }
    }

    .amounts {
        display: flex;
        justify-content: space-evenly;
        gap: $sp2;
        padding: $sp6 0;
    }

    .how-to {
        margin-top: $sp4;
    }

    .custom-tip {
        text-align: center;
        @include font(light, normal, fs-80);

        .custom-tip-amount {
            padding: $sp4 $sp5;
        }
    }
</style>
