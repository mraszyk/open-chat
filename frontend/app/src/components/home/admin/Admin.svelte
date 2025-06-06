<script lang="ts">
    import { platformOperator, ui } from "openchat-client";
    import page from "page";
    import CogOutline from "svelte-material-icons/CogOutline.svelte";
    import Button from "../../Button.svelte";
    import SectionHeader from "../../SectionHeader.svelte";
    import OperatorFunctions from "./OperatorFunctions.svelte";
    import ReviewTranslationCorrections from "./ReviewTranslationCorrections.svelte";

    let selectedTab: "translations" | "operator" = $state("translations");

    function selectTab(tab: "translations" | "operator") {
        selectedTab = tab;
    }
</script>

{#if !$platformOperator}
    <div class="unauthorised">
        <img class="img" src={"/assets/evil-robot.svg"} alt="Unauthorised" />
        <h2>Unauthorised</h2>
        <p>Only platform operators can access the admin area</p>
        <Button onClick={() => page("/")}>Back to safety</Button>
    </div>
{:else}
    <div class="admin">
        <SectionHeader slim border={false}>
            <div class="header">
                <div class="icon">
                    <CogOutline size={ui.iconSize} color={"var(--icon-txt)"} />
                </div>
                <div class="details">
                    <h4 class="name">Admin</h4>
                </div>
            </div>
        </SectionHeader>
        <div class="tabs">
            <div
                tabindex="0"
                role="button"
                onclick={() => selectTab("translations")}
                class:selected={selectedTab === "translations"}
                class="tab">
                Translation Corrections
            </div>
            <div
                tabindex="0"
                role="button"
                onclick={() => selectTab("operator")}
                class:selected={selectedTab === "operator"}
                class="tab">
                Operator functions
            </div>
        </div>
        {#if selectedTab === "translations"}
            <ReviewTranslationCorrections />
        {:else if selectedTab === "operator"}
            <OperatorFunctions />
        {/if}
    </div>
{/if}

<style lang="scss">
    .header {
        display: flex;
        align-items: center;
        gap: $sp3;
    }

    .admin {
        display: flex;
        flex-direction: column;
        height: 100%;
    }

    .tabs {
        display: flex;
        align-items: center;
        @include font(medium, normal, fs-90);
        color: var(--txt-light);
        gap: $sp5;
        border-bottom: 1px solid var(--bd);
        cursor: pointer;
        margin: 0 $sp4 $sp4 $sp4;

        @include mobile() {
            gap: $sp4;
        }

        .tab {
            padding-bottom: 10px;
            margin-bottom: -2px;
            border-bottom: 3px solid transparent;
            white-space: nowrap;
            &.selected {
                color: var(--txt);
                border-bottom: 3px solid var(--txt);
            }
        }
    }

    .unauthorised {
        display: flex;
        justify-content: center;
        align-items: center;
        flex-direction: column;
        height: 100%;
        gap: $sp4;

        .img {
            width: 150px;
            height: 150px;
        }

        h2 {
            @include font(bold, normal, fs-160);
        }
    }
</style>
