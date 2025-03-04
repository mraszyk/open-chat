<script lang="ts">
    import {
        chatPermissionsList,
        communityPermissionsList,
        defaultStringParam,
        messagePermissionsList,
        type SlashCommandParam,
        type SlashCommandSchema,
        ValidationErrors,
    } from "openchat-client";
    import Input from "../Input.svelte";
    import { i18nKey } from "../../i18n/i18n";
    import Legend from "../Legend.svelte";
    import Translatable from "../Translatable.svelte";
    import Checkbox from "../Checkbox.svelte";
    import CommandParamBuilder from "./CommandParamBuilder.svelte";
    import Overlay from "../Overlay.svelte";
    import ModalContent from "../ModalContent.svelte";
    import Link from "../Link.svelte";
    import Button from "../Button.svelte";
    import { mobileWidth } from "../../stores/screenDimensions";
    import SummaryButton from "./SummaryButton.svelte";
    import ButtonGroup from "../ButtonGroup.svelte";
    import ValidatingInput from "./ValidatingInput.svelte";
    import ErrorMessage from "../ErrorMessage.svelte";
    import { togglePermission } from "../../utils/bots";
    import Tabs from "../Tabs.svelte";

    interface Props {
        errors: ValidationErrors;
        errorPath: string;
        command: SlashCommandSchema;
        onAddAnother: () => void;
    }

    let { command = $bindable(), onAddAnother, errors, errorPath }: Props = $props();

    let selectedParam = $state<SlashCommandParam | undefined>(undefined);
    let selectedParamIndex = $state<number | undefined>(undefined);

    function addParameter() {
        command.params.push(defaultStringParam());
        selectedParam = command.params[command.params.length - 1];
        selectedParamIndex = command.params.length - 1;
    }

    function onDeleteParam(param: SlashCommandParam) {
        command.params = command.params.filter((p) => p !== param);
    }

    function onSelectParam(param: SlashCommandParam, index: number) {
        selectedParam = param;
        selectedParamIndex = index;
    }
</script>

{#if selectedParam !== undefined && selectedParamIndex !== undefined}
    <CommandParamBuilder
        errorPath={`${errorPath}_param_${selectedParamIndex}`}
        {errors}
        on:close={() => (selectedParam = undefined)}
        bind:param={selectedParam}
        onAddAnother={addParameter}></CommandParamBuilder>
{/if}

{#snippet chatTab()}
    {#each chatPermissionsList as perm}
        <Checkbox
            id={`chat_permission_${perm}`}
            label={i18nKey(`permissions.${perm}`)}
            checked={command.permissions.chatPermissions.includes(perm)}
            on:change={() => togglePermission(command.permissions, "chatPermissions", perm)}
            align={"start"}>
        </Checkbox>
    {/each}
{/snippet}
{#snippet communityTab()}
    {#each communityPermissionsList as perm}
        <Checkbox
            id={`community_permission_${perm}`}
            label={i18nKey(`permissions.${perm}`)}
            checked={command.permissions.communityPermissions.includes(perm)}
            on:change={() => togglePermission(command.permissions, "communityPermissions", perm)}
            align={"start"}>
        </Checkbox>
    {/each}
{/snippet}
{#snippet messageTab()}
    {#each messagePermissionsList as perm}
        <Checkbox
            id={`message_permission_${perm}`}
            label={i18nKey(`permissions.messagePermissions.${perm}`)}
            checked={command.permissions.messagePermissions.includes(perm)}
            on:change={() => togglePermission(command.permissions, "messagePermissions", perm)}
            align={"start"}>
        </Checkbox>
    {/each}
{/snippet}

<Overlay>
    <ModalContent on:close>
        <div slot="header">
            <Translatable resourceKey={i18nKey("bots.builder.commandLabel", { name: command.name })}
            ></Translatable>
        </div>
        <div slot="body">
            <section>
                <Legend
                    required
                    label={i18nKey("bots.builder.commandNameLabel")}
                    rules={i18nKey("bots.builder.nameRules")}></Legend>
                <ValidatingInput
                    autofocus
                    error={errors.get(`${errorPath}_name`)}
                    minlength={3}
                    maxlength={25}
                    invalid={errors.has(`${errorPath}_name`)}
                    placeholder={i18nKey("bots.builder.commandNamePlaceholder")}
                    bind:value={command.name} />
            </section>

            <section>
                <Legend
                    label={i18nKey("bots.builder.commandDescLabel")}
                    rules={i18nKey("bots.builder.optional")}></Legend>
                <Input
                    minlength={3}
                    maxlength={200}
                    placeholder={i18nKey("bots.builder.commandDescPlaceholder")}
                    bind:value={command.description} />
            </section>

            <section>
                <Legend
                    label={i18nKey("bots.builder.commandPermissionsLabel")}
                    rules={i18nKey("bots.builder.commandPermissionsDesc")}></Legend>

                <Tabs
                    initialIndex={2}
                    tabs={[
                        {
                            title: i18nKey("bots.builder.permScopeCommunity"),
                            snippet: communityTab,
                        },
                        {
                            title: i18nKey("bots.builder.permScopeChat"),
                            snippet: chatTab,
                        },
                        {
                            title: i18nKey("bots.builder.permScopeMessage"),
                            snippet: messageTab,
                        },
                    ]}></Tabs>
            </section>

            <section>
                {#each command.params as param, i}
                    <SummaryButton
                        valid={!errors.has(`${errorPath}_param_${i}`)}
                        onSelect={() => onSelectParam(param, i)}
                        onDelete={() => onDeleteParam(param)}
                        resourceKey={i18nKey("bots.builder.paramLabel", { name: param.name })}
                    ></SummaryButton>
                {/each}
            </section>

            {#if errors.has(`${errorPath}_duplicate_params`)}
                <ErrorMessage>{errors.get(`${errorPath}_duplicate_params`)}</ErrorMessage>
            {/if}

            <Link on:click={addParameter} underline={"never"}>
                <Translatable resourceKey={i18nKey("bots.builder.addParam")}></Translatable>
            </Link>
        </div>

        <div let:onClose slot="footer" class="footer">
            <ButtonGroup>
                <Button secondary on:click={onAddAnother} small={!$mobileWidth} tiny={$mobileWidth}>
                    <Translatable resourceKey={i18nKey("bots.builder.addAnother")} />
                </Button>
                <Button on:click={onClose} small={!$mobileWidth} tiny={$mobileWidth}>
                    <Translatable resourceKey={i18nKey("close")} />
                </Button>
            </ButtonGroup>
        </div>
    </ModalContent>
</Overlay>

<style lang="scss">
    section {
        margin-bottom: $sp4;
    }
</style>
