<script lang="ts">
    import type { DataContent, OpenChat } from "openchat-client";
    import { getContext } from "svelte";

    interface Props {
        banner: DataContent;
        square?: boolean;
        intersecting?: boolean;
        children?: import("svelte").Snippet;
    }

    let { banner, square = false, intersecting = false, children }: Props = $props();

    const client = getContext<OpenChat>("client");

    let style = $derived(
        intersecting ? `background-image: url(${client.communityBannerUrl(banner)})` : "",
    );
</script>

<div class:square class="banner" {style}>
    {@render children?.()}
</div>

<style lang="scss">
    .banner {
        position: relative;
        background-size: cover;
        background-position: center center;
        border-radius: var(--card-rd) var(--card-rd) 0 0;
        padding-bottom: 50%; // forces 2:1 aspect ratio

        &.square {
            border-radius: 0;
        }
    }
</style>
