<script lang="ts">
    import { communityThemes, currentTheme, themes } from "../../theme/themes";
    import Feature from "./Feature.svelte";

    import { ui } from "openchat-client";
    import { menuStore } from "../../stores/menu";

    let scrollTop = $state(0);
    let phoneBorder = 5;
    let windowHeight = $state(window.innerHeight);
    let menuHeight = ui.toPixel(5);

    // all the crazy calculations
    let sectionHeight = $derived(ui.availableHeight);
    let phoneHeight = $derived(ui.mobileWidth ? ui.availableHeight * 0.7 : 750);
    let phoneTop = $derived((sectionHeight - phoneHeight) / 2 + menuHeight);
    let phoneWidth = $derived(phoneHeight * 0.5333);
    let cssHeight = $derived(phoneHeight + phoneBorder * 2);
    let cssWidth = $derived(phoneWidth + phoneBorder * 2);
    let scrollOffset = $derived((sectionHeight - cssHeight) / 2);

    const black = "#242834";

    function onScroll() {
        menuStore.hideMenu();
        scrollTop = window.scrollY;
    }

    function clamp(n: number) {
        if (n < 0) return 0;
        if (n > phoneHeight) return phoneHeight;
        return n;
    }

    const screenshotMap: Record<string, { url: string; alt: string }[]> = $state({});

    [...communityThemes, themes.dark, themes.white].forEach((theme) => {
        screenshotMap[theme.name] = [
            {
                url: `/assets/screenshots/features/communities_${theme.mode}.webp`,
                alt: "communities",
            },
            {
                url: `/assets/screenshots/features/mobilefirst_${theme.mode}.webp`,
                alt: "mobile first",
            },
            {
                url: `/assets/screenshots/features/creategroup_${theme.mode}.webp`,
                alt: "create group",
            },
            {
                url: `/assets/screenshots/features/permissions_${theme.mode}.webp`,
                alt: "permissions",
            },
            { url: `/assets/screenshots/features/organise_${theme.mode}.webp`, alt: "organise" },
            { url: `/assets/screenshots/features/profile_${theme.mode}.webp`, alt: "user profile" },
            {
                url: `/assets/screenshots/features/message_${theme.mode}.gif`,
                alt: "sending messages",
            },
            { url: `/assets/screenshots/features/search_${theme.mode}.gif`, alt: "searching" },
            { url: `/assets/screenshots/features/voting_${theme.mode}.webp`, alt: "voting" },
        ];
    });

    let screenshots = $derived(screenshotMap[$currentTheme.name] ?? []);
</script>

<svelte:window bind:innerHeight={windowHeight} onscroll={onScroll} />

<div
    class="phone"
    style={`top: ${phoneTop}px; height: ${cssHeight}px; width: ${cssWidth}px; transform: translateX(${cssWidth}px)`}>
    {#each screenshots as screenshot, i}
        <div
            style={`height: ${
                i === 0 ? phoneHeight : clamp(scrollTop - (scrollOffset + sectionHeight * (i - 1)))
            }px`}
            class="feature-img-container">
            <img class="feature-img" src={screenshot.url} alt={screenshot.alt} />
        </div>
    {/each}
</div>

<div class="content">
    <Feature height={sectionHeight} backgroundColor={"transparent"} title={"Communities"}>
        <p>
            Not just a chat app - OpenChat is a Community building power house. Find communities
            that speak to you. Create a community of your own - you are in complete control using
            our powerful system of roles, permissions and channels.
        </p>
        <p>Discord meets Slack in a decentralized package.</p>
    </Feature>

    <Feature
        height={sectionHeight}
        backgroundColor={"#FF005C"}
        color={"#ffffff"}
        title={"Mobile first"}>
        <p>
            A chat app should be used on the go and so OpenChat was designed from the beginning to
            work well first and foremost on your mobile device.
        </p>
        <p>
            The user interface will respond to give a seamless experience on devices of any size
            from mobile to desktop.
        </p>
    </Feature>

    <Feature height={sectionHeight} backgroundColor={"#FEC000"} color={black} title={"Groups"}>
        <p>
            Create private groups with friends and family to coordinate and chat together. With a
            private group, you have full control over who is the group.
        </p>
        <p>Or create a public group and share it with the world.</p>
    </Feature>

    <Feature height={sectionHeight} backgroundColor={"#08AEDB"} color={black} title={"Permissions"}>
        <p>
            Permissions are assigned to different types of users. As a community or group owner you
            will decide who gets owner, admin or moderator privileges. This allows delegate
            responsibility as you grow your space.
        </p>
    </Feature>

    <Feature
        height={sectionHeight}
        backgroundColor={"#673BB7"}
        color={"#ffffff"}
        title={"Organise"}>
        <p>Add any chat to your favourites for easy access.</p>
        <p>Pin any chat to the top of a chat list.</p>
        <p>Arrange communities in any order that suits you.</p>
    </Feature>

    <Feature
        height={sectionHeight}
        backgroundColor={"#05B09F"}
        color={black}
        title={"User profile"}>
        <p>Configure your personal information, UI settings and chat settings at any time.</p>

        <p>Manage your crypt accounts and account storage.</p>

        <p>View your own personl stats. Get messaging!</p>
    </Feature>

    <Feature
        height={sectionHeight}
        backgroundColor={"#FF8541"}
        color={"#ffffff"}
        title={"Sending messages"}>
        <p>
            Sending messages is the heart of any chat app. OpenChat provides all of the features
            that you would expect and adds a few unique capabilities of its own.
        </p>
    </Feature>

    <Feature height={sectionHeight} backgroundColor={"#FF005C"} color={"#ffffff"} title={"Search"}>
        <p>Search for Communities in the community explorer.</p>
        <p>Search for global groups from the group chats section.</p>
        <p>Search for any user in from the direct chats section.</p>

        <p>You can also search for messages within any selected chat.</p>
    </Feature>

    <Feature height={sectionHeight} backgroundColor={"transparent"} title={"Proposal voting"}>
        <p>
            A unique feature of OpenChat is that it allows you to vote directly on NNS and <em
                >any</em>
            SNS proposals.
        </p>

        <p>
            Simply register your OpenChat account as a hotkey for the neuron that you wish to vote
            with and join the relevant public group.
        </p>
    </Feature>
</div>

<style lang="scss">
    p {
        @include font(light, normal, fs-120, 28);
        margin-bottom: toRem(24);
    }

    .phone {
        pointer-events: none;
        overflow: hidden;
        display: block;
        position: fixed;
        right: 40%;
        border: 5px solid var(--landing-phone-bd);
        border-radius: toRem(18);
        @include box-shadow(3);
        @include z-index("phone");
    }

    .feature-img-container {
        display: block;
        position: absolute;
        bottom: 0;
        left: 0;
        @include z-index("phone-image");
        width: 100%;

        .feature-img {
            width: 100%;
            max-width: 100%;
            height: 100%;
            max-height: 100%;
            object-fit: cover;
            object-position: bottom;
        }
    }

    .content {
        position: relative;
        @include z-index("features");
        padding: 0;
    }
</style>
