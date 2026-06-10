<script>
    import Report from "./components/Report.svelte";
    import Page from "./components/Page.svelte";

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
</script>

<Page>
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
            <b class="card-footer-item"
                ><div class="control">
                    <button on:click={openOptions} class="button"
                        >Open settings</button
                    >
                </div>
            </b>

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
