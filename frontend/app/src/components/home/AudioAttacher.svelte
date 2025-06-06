<script lang="ts">
    import { ui, type AudioContent, type OpenChat } from "openchat-client";
    import { getContext, onMount } from "svelte";
    import { _ } from "svelte-i18n";
    import Microphone from "svelte-material-icons/Microphone.svelte";
    import RadioboxMarked from "svelte-material-icons/RadioboxMarked.svelte";
    import { i18nKey } from "../../i18n/i18n";
    import { toastStore } from "../../stores/toast";
    import HoverIcon from "../HoverIcon.svelte";

    const client = getContext<OpenChat>("client");

    interface Props {
        recording?: boolean;
        percentRecorded?: number;
        mimeType: string;
        supported: boolean;
        onAudioCaptured: (content: AudioContent) => void;
    }

    let {
        recording = $bindable(false),
        percentRecorded = $bindable(0),
        mimeType,
        supported = $bindable(),
        onAudioCaptured,
    }: Props = $props();

    percentRecorded;

    let mediaRecorder: MediaRecorder | undefined;

    onMount(() => {
        if (supported) {
            // TODO - there are problems with this whole thing on safari.
            // in particular, there is no access to the permissions api
            if ("permissions" in navigator) {
                navigator.permissions
                    //@ts-ignore
                    .query({ name: "microphone" })
                    .then(function (result) {
                        if (result.state === "denied") {
                            // if they already said no, don't be rude
                            supported = false;
                        }
                    })
                    .catch((_err) => {
                        console.log(
                            "unable to check microphone permissions (probably unsupported)",
                        );
                    });
            }
        }
    });

    function stopRecording() {
        if (mediaRecorder && mediaRecorder.state === "recording") {
            mediaRecorder.stop();
        }
    }

    function toggle() {
        if (recording) {
            stopRecording();
        } else {
            record();
        }
    }

    function record() {
        if (supported) {
            navigator.mediaDevices
                .getUserMedia({ audio: true, video: false })
                .then((stream) => {
                    recording = true;
                    const recordedChunks: Blob[] = [];
                    const maxSizes = client.maxMediaSizes();
                    let totalSize = 0;
                    let truncated = false;
                    percentRecorded = 0;
                    mediaRecorder = new MediaRecorder(stream, { mimeType });

                    mediaRecorder.addEventListener("dataavailable", (e) => {
                        if (e.data.size > 0) recordedChunks.push(e.data);
                        totalSize += e.data.size;
                        percentRecorded = (totalSize / maxSizes.audio) * 100;
                        if (totalSize >= maxSizes.audio) {
                            truncated = true;
                            stopRecording();
                        }
                    });

                    mediaRecorder.addEventListener("stop", async () => {
                        const data = await new Blob(recordedChunks).arrayBuffer();
                        mediaRecorder = undefined;
                        recording = false;
                        if (truncated) {
                            // let the user know if we stopped recording prematurely
                            toastStore.showFailureToast(i18nKey("maxAudioSize"));
                        }
                        onAudioCaptured({
                            kind: "audio_content",
                            mimeType: mimeType,
                            blobData: new Uint8Array(data),
                            blobUrl: client.dataToBlobUrl(data, mimeType),
                        });
                        stream.getTracks().forEach((track) => track.stop());
                    });

                    mediaRecorder.start(200);
                })
                .catch(() => (supported = false)); //catch the case where the user denies access
        }
    }
</script>

{#if supported}
    <div onclick={toggle}>
        <HoverIcon title={recording ? $_("stopRecording") : $_("recordAudioMessage")}>
            {#if recording}
                <RadioboxMarked size={ui.iconSize} color={"red"} />
            {:else}
                <Microphone size={ui.iconSize} color={"var(--icon-txt)"} />
            {/if}
        </HoverIcon>
    </div>
{/if}
