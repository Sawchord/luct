<script>
    import Report from "./components/Report.svelte";

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
        const report = await browser.runtime.sendMessage({ tabId });

        if (report) {
            let certs = new Map();

            for (const [url, rep] of report.urls) {
                if (!rep.report) {
                    continue;
                }

                let fingerprint = rep.report.fingerprint;
                let existing_entry = certs.get(fingerprint);
                if (existing_entry) {
                    existing_entry.urls.push(url);
                    if (
                        existing_entry.status === "safe" &&
                        rep.status !== "safe"
                    ) {
                        existing_entry.status = rep.status;
                    }
                    certs.set(fingerprint, existing_entry);
                } else {
                    certs.set(fingerprint, {
                        report: rep.report,
                        urls: [],
                        status: rep.status,
                    });
                }
            }

            reports = Array.from(certs, ([_name, value]) => value);
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
</script>

{#await update_content()}
    <p>Loading</p>
{:then report}
    {#each reports as report}
        <Report {report}></Report>
    {/each}
{/await}
