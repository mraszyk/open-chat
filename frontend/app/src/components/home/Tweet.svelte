<script lang="ts">
    import { ui } from "openchat-client";
    import { onMount } from "svelte";
    import { currentTheme } from "../../theme/themes";

    interface Props {
        intersecting: boolean;
        tweetId: string;
    }

    let { intersecting, tweetId }: Props = $props();

    let tweetWrapper: HTMLDivElement | undefined = $state();
    let supported = $state(false);

    let rendering: Promise<any> | undefined = $state(undefined);

    onMount(() => {
        supported = (<any>window).twttr !== undefined;
    });

    $effect(() => {
        if (
            intersecting &&
            !ui.eventListScrolling &&
            tweetWrapper !== undefined &&
            !rendering &&
            supported
        ) {
            tweetWrapper.innerHTML = "";

            rendering = (<any>window).twttr?.widgets.createTweet(tweetId, tweetWrapper, {
                conversation: "none",
                theme: $currentTheme.mode,
            }) as Promise<any>;

            rendering.catch((err: any) => {
                console.log("Failed to render tweet: ", err);
                rendering = undefined;
            });
        }
    });
</script>

<div bind:this={tweetWrapper}></div>
