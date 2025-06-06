<script lang="ts">
    import Roadmap from "./RoadmapOverview.svelte";
    import Intro from "./Intro.svelte";
    import Team from "./Team.svelte";
    import SellingPoints from "./SellingPoints.svelte";
    import BragBox from "./BragBox.svelte";
    import ArrowLink from "../ArrowLink.svelte";
    import SignInWithMagicLink from "../SignInWithMagicLink.svelte";
    import { onMount } from "svelte";
    import Overlay from "../Overlay.svelte";
    import { pathState } from "openchat-client";

    let showSignInWithMagicLinkModal = false;

    onMount(() => {
        if (pathState.querystring.has("auth")) {
            showSignInWithMagicLinkModal = true;
        }
    });
</script>

<div class="content">
    <Intro />

    <div class="headline">
        <h2>
            OpenChat users can send messages to each other containing tokens like ICP and ckBTC.
        </h2>
        <p>
            OpenChat is governed by its community as a DAO and has its own CHAT token. CHAT tokens
            are given as rewards to users to turbo-charge growth and to create a team of millions of
            advocates allowing OpenChat to become a viable challenger to centralized big tech
            competitors!
        </p>
        <div class="features">
            <ArrowLink url={"/features"} color={"#23A2EE"}>View Features</ArrowLink>
        </div>
    </div>

    <SellingPoints />

    <Roadmap />
</div>

<BragBox />

<div class="content">
    <Team />
</div>

{#if showSignInWithMagicLinkModal}
    <Overlay>
        <SignInWithMagicLink onClose={() => (showSignInWithMagicLinkModal = false)} />
    </Overlay>
{/if}

<style lang="scss">
    .headline {
        display: grid;
        gap: $sp5;
        grid-template-columns: 4fr 1fr;
        grid-template-areas:
            "title ."
            "blurb features";
        margin-bottom: toRem(80);
        align-items: flex-end;

        @include mobile() {
            grid-template-columns: 1fr;
            grid-template-areas:
                "title"
                "blurb"
                "features";
        }

        h2 {
            grid-area: title;
            @include font(medium, normal, fs-250, 91);
            margin-bottom: toRem(48);
            text-align: left;

            @include mobile() {
                @include font(medium, normal, fs-220);
                line-height: toRem(51);
                margin-bottom: toRem(16);
            }
        }
        p {
            grid-area: blurb;
            margin-bottom: 0;
            color: var(--landing-txt-light);
            max-width: 80%;
            @include font(book, normal, fs-120, 28);

            @include mobile() {
                max-width: 100%;
                margin-bottom: toRem(40);
            }
        }
        .features {
            grid-area: features;
            justify-self: flex-end;
            @include mobile() {
                justify-self: flex-start;
            }
        }
    }
    .content {
        @include lp-content-padding();

        @include mobile() {
            padding: 0 toRem(24);
        }
    }
</style>
