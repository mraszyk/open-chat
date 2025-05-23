<script lang="ts">
    import Loading from "../../Loading.svelte";
    import ThreadPreviewComponent from "./ThreadPreview.svelte";
    import { getContext } from "svelte";
    import { toastStore } from "../../../stores/toast";
    import type { OpenChat, ThreadPreview } from "openchat-client";
    import {
        selectedChatId,
        threadsByChatStore as threadsByChat,
        serverChatSummariesStore,
    } from "openchat-client";
    import { i18nKey } from "../../../i18n/i18n";

    const client = getContext<OpenChat>("client");

    let threads: ThreadPreview[] = $state([]);
    let observer: IntersectionObserver = new IntersectionObserver(() => {});
    let loading = $state(false);
    let initialised = $state(false);

    $effect(() => {
        loading = true;
        client
            .threadPreviews($selectedChatId, $threadsByChat, $serverChatSummariesStore)
            .then((t) => {
                threads = t;
                initialised = true;
            })
            .catch((err) => {
                toastStore.showFailureToast(i18nKey("thread.previewFailure"));
                client.logError("Unable to load thread previews: ", err);
            })
            .finally(() => (loading = false));
    });
</script>

<div class="threads">
    {#if loading && !initialised}
        <Loading />
    {:else}
        {#each threads as thread (`${thread.rootMessage.index}_${thread.rootMessage.event.messageId}`)}
            <ThreadPreviewComponent {observer} {thread} />
        {/each}
    {/if}
</div>

<style lang="scss">
    .threads {
        height: 100%;
        overflow-x: hidden;
        padding: 0 $sp4;
        @include nice-scrollbar();
        @include mobile() {
            padding: 0 toRem(10);
        }
    }
</style>
