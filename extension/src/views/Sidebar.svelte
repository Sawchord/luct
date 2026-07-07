<script>
    import Report from "../components/report/Report.svelte";
    import Page from "../components/Page.svelte";
    import LuctLogo from "../components/LuctLogo.svelte";

    let windowId;
    let tabId;
    let reports = [];

    browser.windows.getCurrent({ populate: true }).then(async (windowInfo) => {
        windowId = windowInfo.id;

        let tabs = await browser.tabs.query({ windowId, active: true });
        tabId = tabs[0].id;
        update_content();
    });

    browser.tabs.onActivated.addListener((tab) => {
        tabId = tab.tabId;
        update_content();
    });

    browser.runtime.onMessage.addListener((_message) => {
        update_content();
    });

    async function update_content() {
        try {
            let response = await browser.runtime.sendMessage({ tabId });

            if (response) {
                reports = Array.from(response.reports, ([_, value]) => value);
            } else {
                reports = [];
            }
        } catch (err) {
            console.log(
                "Updating content failed because background script has not started yet",
            );
        }
    }

    // TODO: Render testdata with warning if not connected
    // async function get_testdata() {
    //     let data = await fetch("../testdata/test-report.json");
    //     console.log(data);
    //     let json = await data.json();
    //     console.log(json);
    //     return json;
    // }

    function openOptions() {
        browser.runtime.openOptionsPage();
    }

    function openDashboard() {
        window.open("/dashboard.html");
    }
</script>

<Page>
    <div slot="header" class="card">
        <nav class="navbar navbar-extra" aria-label="main navigation">
            <div class="navbar-brand">
                <p class="navbar-item">
                    <LuctLogo></LuctLogo>
                </p>
            </div>
            <div class="navbar-end navbar-end-extra">
                <div class="navbar-item">
                    <div class="buttons">
                        <button title="Open Dashboard" on:click={openDashboard}>
                            <span class="icon is-large">
                                <i
                                    class="fa fa-lg fa-bar-chart"
                                    aria-label="options"
                                ></i>
                            </span>
                        </button>
                        <button title="Open Settings" on:click={openOptions}>
                            <span class="icon is-large">
                                <i class="fa fa-lg fa-cog" aria-label="options"
                                ></i>
                            </span>
                        </button>
                    </div>
                </div>
            </div>
        </nav>
    </div>
    <div slot="content">
        {#await update_content()}
            <p>Loading</p>
        {:then report}
            {#each reports as report}
                <Report {report}></Report>
            {/each}
        {/await}
    </div>
    <div slot="footer">
        <p>
            <b class="card-footer-item">
                <span
                    >Built with 🤎 by <a
                        href="https://github.com/Sawchord"
                        class="link">Sawchord</a
                    ></span
                >
            </b>
        </p>
    </div>
</Page>

<style lang="sass">
.navbar-extra
    display: flex

.navbar-end-extra
    margin-inline-start: auto
</style>
