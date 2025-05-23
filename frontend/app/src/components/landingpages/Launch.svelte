<script lang="ts">
    import type { OpenChat } from "openchat-client";
    import { app, routeForScope } from "openchat-client";
    import { getContext } from "svelte";

    const client = getContext<OpenChat>("client");

    interface Props {
        rootPath?: any;
        text?: string;
        login?: boolean;
    }

    let {
        rootPath = routeForScope(client.getDefaultScope()),
        text = "Launch app",
        login = false,
    }: Props = $props();

    let url = $derived(app.identityState.kind === "logged_in" ? rootPath : "/communities");
    let busy = $derived(
        app.identityState.kind === "logging_in" || app.identityState.kind === "loading_user",
    );
</script>

{#if login}
    <div
        class:loading={busy}
        role="button"
        tabindex="0"
        onclick={() => client.login()}
        class="launch">
        {#if !busy}
            {text}
        {/if}
    </div>
{:else}
    <a href={url} class="launch">{text}</a>
{/if}

<style lang="scss">
    .launch {
        display: inline-block;
        transition: background-color ease-in-out 200ms;
        color: var(--button-txt);
        background-color: var(--button-bg);
        border: none;
        border-radius: toRem(4);
        cursor: pointer;
        text-decoration: none;
        min-height: 45px;
        min-width: 150px;
        text-align: center;
        @include font(bold, normal, fs-100);
        padding: toRem(12) toRem(16) toRem(12) toRem(16);

        &:hover {
            background-color: var(--button-hv);
        }

        @include mobile() {
            @include font(bold, normal, fs-120);
            padding: toRem(16) toRem(20);
            width: 100%;
        }

        &.loading {
            @include loading-spinner(
                1em,
                0.5em,
                var(--button-spinner),
                "/assets/plain-spinner.svg"
            );
        }
    }
</style>
