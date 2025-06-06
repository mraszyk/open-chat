<script lang="ts">
    import type { ChatSummary, OpenChat } from "openchat-client";
    import { filteredProposalsStore, proposalTopicsStore, ui } from "openchat-client";
    import { getContext } from "svelte";
    import { _ } from "svelte-i18n";
    import Close from "svelte-material-icons/Close.svelte";
    import { i18nKey } from "../../i18n/i18n";
    import {
        proposalActionCategories,
        type ProposalActionCategory,
    } from "../../stores/proposalSections";
    import { OC_GOVERNANCE_CANISTER_ID } from "../../utils/sns";
    import Checkbox from "../Checkbox.svelte";
    import CollapsibleCard from "../CollapsibleCard.svelte";
    import HoverIcon from "../HoverIcon.svelte";
    import LinkButton from "../LinkButton.svelte";
    import SectionHeader from "../SectionHeader.svelte";
    import Translatable from "../Translatable.svelte";

    interface Props {
        selectedChat: ChatSummary;
        onClose: () => void;
    }

    let { selectedChat, onClose }: Props = $props();

    type SectionLabels = Record<ProposalActionCategory, string>;

    const sectionLabels: SectionLabels = {
        all: "",
        unknown: "proposal.unknownActionCategory",
        builtIn: "proposal.builtInAction",
        userIndex: "proposal.userIndexAction",
        groupIndex: "proposal.groupIndexAction",
        notifications: "proposal.notificationsAction",
        proposalsBot: "proposal.proposalsBotAction",
        storageIndex: "proposal.storageIndexAction",
        cyclesDispenser: "proposal.cyclesDispenserAction",
        registry: "proposal.registryAction",
        neuronController: "proposal.neuronControllerAction",
        openchatInstaller: "proposal.openchatInstallerAction",
    };

    const client = getContext<OpenChat>("client");
    let topics = $derived([...$proposalTopicsStore]);
    let groupTopics = $derived(
        selectedChat.kind !== "direct_chat" &&
            selectedChat.subtype?.governanceCanisterId === OC_GOVERNANCE_CANISTER_ID,
    );

    let grouped = $derived([
        ...client.groupBy(topics, ([id, _]) => {
            if (!groupTopics) return "all";

            if (id < 1000) {
                return "builtIn";
            } else if (id < 2000) {
                return "userIndex";
            } else if (id < 3000) {
                return "groupIndex";
            } else if (id < 4000) {
                return "notifications";
            } else if (id < 5000) {
                return "proposalsBot";
            } else if (id < 6000) {
                return "storageIndex";
            } else if (id < 7000) {
                return "cyclesDispenser";
            } else if (id < 8000) {
                return "registry";
            } else if (id < 9000) {
                return "neuronController";
            } else if (id < 10000) {
                return "openchatInstaller";
            } else {
                return "unknown";
            }
        }),
    ]);

    function kebab(name: string): string {
        return `topic_${name.toLowerCase().split(" ").join("_")}`;
    }
</script>

<SectionHeader shadow flush={ui.mobileWidth}>
    <h4><Translatable resourceKey={i18nKey("proposal.filter")} /></h4>
    <span title={$_("close")} class="close" onclick={onClose}>
        <HoverIcon>
            <Close size={ui.iconSize} color={"var(--icon-txt)"} />
        </HoverIcon>
    </span>
</SectionHeader>

<div class="proposal-filters">
    <div class="controls">
        <LinkButton onClick={client.enableAllProposalFilters} underline={"hover"}
            ><Translatable resourceKey={i18nKey("proposal.enableAll")} /></LinkButton>
        <LinkButton
            onClick={() => client.disableAllProposalFilters(topics.map(([id]) => id))}
            underline={"hover"}
            ><Translatable resourceKey={i18nKey("proposal.disableAll")} /></LinkButton>
    </div>
    {#each grouped as [category, topicsInCategory]}
        {#if groupTopics}
            <CollapsibleCard
                onToggle={() => proposalActionCategories.toggle(category)}
                open={$proposalActionCategories[category]}
                headerText={i18nKey(sectionLabels[category])}>
                {#each topicsInCategory as [id, label]}
                    <div class="toggle">
                        <Checkbox
                            id={kebab(label)}
                            onChange={() => client.toggleProposalFilter(id)}
                            label={i18nKey(label)}
                            checked={!$filteredProposalsStore?.hasFilter(id)} />
                    </div>
                {/each}
            </CollapsibleCard>
        {:else}
            {#each topicsInCategory as [id, label]}
                <div class="toggle">
                    <Checkbox
                        id={kebab(label)}
                        onChange={() => client.toggleProposalFilter(id)}
                        label={i18nKey(label)}
                        checked={!$filteredProposalsStore?.hasFilter(id)} />
                </div>
            {/each}
        {/if}
    {/each}
</div>

<style lang="scss">
    h4 {
        flex: 1;
        margin: 0;
        @include font-size(fs-120);
    }
    .close {
        flex: 0 0 30px;
    }

    .toggle {
        margin-bottom: $sp4;
    }

    .proposal-filters {
        background-color: var(--bg);
        padding: $sp4;
        padding-bottom: 0;
        @include nice-scrollbar();

        @include mobile() {
            height: 100%;
        }
    }

    .controls {
        display: flex;
        gap: $sp4;
        align-items: center;
        margin-bottom: $sp4;
    }
</style>
