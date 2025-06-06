<script lang="ts">
    import { AvatarSize, OpenChat, ui, userStore, type CommandArg } from "openchat-client";
    import type { BotContextCommand } from "openchat-shared";
    import { getContext } from "svelte";
    import CogOutline from "svelte-material-icons/CogOutline.svelte";
    import Avatar from "../Avatar.svelte";
    import Typing from "../Typing.svelte";
    import Markdown from "../home/Markdown.svelte";
    import Tooltip from "../tooltip/Tooltip.svelte";

    const client = getContext<OpenChat>("client");

    interface Props {
        botCommand: BotContextCommand;
        botName: string;
        finalised: boolean;
    }

    let { botCommand, finalised }: Props = $props();
    let MAX_COMMAND_LENGTH = $derived(ui.mobileWidth ? 50 : 150);
    let paramValues = $derived(botCommand.args.map(paramValue));
    let paramsLength = $derived(paramValues.reduce((total, p) => total + p.length, 0));
    let paramMode: "truncated" | "full" = $derived(
        paramsLength > MAX_COMMAND_LENGTH ? "truncated" : "full",
    );
    let text = $derived.by(() => {
        if (paramMode === "truncated") {
            return `@UserId(${botCommand.initiator}) used **/${botCommand.name}**${
                botCommand.args.length > 0 ? " with " : ""
            }`;
        } else {
            return `@UserId(${botCommand.initiator}) used **/${botCommand.name}**`;
        }
    });
    let user = $derived($userStore.get(botCommand.initiator));

    function paramValue(param: CommandArg): string {
        switch (param.kind) {
            case "boolean":
                return param.value?.toString() ?? "false";
            case "integer":
            case "decimal":
                return param.value?.toString() ?? "null";
            case "string":
                return param.value ?? "null";
            case "user":
                return param.userId ? $userStore.get(param.userId)?.username ?? "null" : "null";
            case "dateTime":
                return param.value
                    ? client.toDatetimeString(new Date(Number(param.value)))
                    : "null";
        }
    }
</script>

<div class="bot-context">
    {#if user}
        <Avatar url={client.userAvatarUrl(user)} userId={user.userId} size={AvatarSize.Tiny} />
    {/if}
    <Markdown {text} />
    {#if botCommand.args.length > 0}
        {#if paramMode === "truncated"}
            <Tooltip position="right" align="middle">
                <div class="cog">
                    <CogOutline size={"1.2em"} color={"var(--icon-txt)"} />
                </div>
                {#snippet popupTemplate()}
                    <div class="command-params">
                        {#each botCommand.args as param}
                            <div class="param">
                                <div class="name">{param.name}:</div>
                                <div class="value">{paramValue(param)}</div>
                            </div>
                        {/each}
                    </div>
                {/snippet}
            </Tooltip>
        {:else}
            {#each paramValues as param}
                <div class="inline-param">{param}</div>
            {/each}
        {/if}
    {/if}
    {#if !finalised}
        <Typing />
    {/if}
</div>

<style lang="scss">
    .bot-context {
        @include font(light, normal, fs-70);
        color: var(--txt-light);
        display: flex;
        gap: $sp2;
        align-items: center;
        @include ellipsis();

        :global(.markdown-wrapper) {
            @include ellipsis();
            flex-shrink: 0;
        }

        :global(.avatar.tiny) {
            flex: 0 0 toRem(20);
        }

        .cog {
            display: flex;
        }
    }

    .inline-param {
        // background-color: #efefef;
        padding: 0 $sp2;
        border-radius: $sp2;
        border: 1px solid var(--bd);
        @include font(light, normal, fs-60);
    }

    :global(.command-params) {
        word-wrap: unset;

        .param {
            display: flex;
            align-items: start;
            text-align: left;
            gap: $sp2;
            flex-wrap: nowrap;
            flex-direction: column;

            .value {
                @include font(bold, normal, fs-50);
            }
        }
    }
</style>
