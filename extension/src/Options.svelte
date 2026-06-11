<script>
    import Page from "./components/Page.svelte";

    let settings = JSON.parse(window.localStorage.getItem("settings"));

    async function store_and_reload() {
        const json_settings = JSON.stringify(settings);
        window.localStorage.setItem("settings", json_settings);

        const _report = await browser.runtime.sendMessage("reload");
    }

    function download(filename, data) {
        var element = document.createElement("a");
        element.setAttribute(
            "href",
            "data:text/plain;charset=utf-8," + encodeURIComponent(data),
        );
        element.setAttribute("download", filename);
        element.style.display = "none";

        element.click();
    }

    function export_store() {
        var output = [];

        for (let key of Object.keys(window.localStorage)) {
            try {
                let value = JSON.parse(window.localStorage.getItem(key));
                output.push([key, value]);
            } catch (error) {
                console.log("Failed to export key: " + key);
                console.log("Failed value: " + value);
                console.log(error);
            }
        }
        let output_string = JSON.stringify(output);

        download("luct.json", output_string);
    }

    function load_store(input) {
        const data = JSON.parse(input);

        window.localStorage.clear();
        for (let [key, value] of data) {
            window.localStorage.setItem(key, JSON.stringify(value));
        }
    }

    function import_store() {
        const input = document.createElement("input");
        input.type = "file";

        input.onchange = (event) => {
            const file = event.target.files[0];
            const reader = new FileReader();

            reader.addEventListener("load", () => {
                const data = reader.result;
                load_store(data);
                console.log("Import complete");
            });
            reader.readAsText(file);
        };
        input.click();
    }
</script>

<Page>
    <div slot="content" class="card">
        <header class="card-header">
            <p class="card-header-title">luCT Settings</p>
            <button class="card-header-icon" aria-label="more options">
                <span class="icon is-large">
                    <i class="fa fa-lg fa-cog" aria-hidden="true"></i>
                </span>
            </button>
        </header>

        <div class="card-content">
            <label class="checkbox">
                <input
                    bind:checked={settings.validate_cert_chain}
                    type="checkbox"
                />
                Validate certificate chain
            </label>

            <div class="field">
                <!-- svelte-ignore a11y-label-has-associated-control -->
                <label class="label">Oblivious TLS proxy</label>
                <label class="checkbox">
                    <input bind:checked={settings.use_otlsp} type="checkbox" />
                    Use oblivious TLS proxy
                </label>
                <div>
                    <div class="control">
                        <input
                            bind:value={settings.otlsp_url}
                            class="input"
                            type="text"
                            placeholder=""
                        />
                    </div>
                    <p class="help">
                        Full url to the OTLSP endpoint. E.g.
                        "https://node.luct.dev/otlsp"
                    </p>
                </div>
                <div>
                    <div class="control">
                        <input
                            bind:value={settings.otlsp_connection_timeout}
                            class="input"
                            type="number"
                            placeholder=""
                        />
                    </div>
                    <p class="help">
                        Time (in seconds) after which a connectino to an
                        oblivious TLS proxy is considered stale
                    </p>
                </div>
            </div>

            <div class="field">
                <!-- svelte-ignore a11y-label-has-associated-control -->
                <label class="label">STH freshness threshold (in seconds)</label
                >
                <div class="control">
                    <input
                        bind:value={settings.sth_freshness_threshold}
                        class="input"
                        type="number"
                        placeholder=""
                    />
                </div>
                <p class="help">
                    STHs younger than this are considered "fresh", older ones
                    "mature". This in important in the luCT policy evaluation.
                </p>
            </div>

            <div class="field">
                <!-- svelte-ignore a11y-label-has-associated-control -->
                <label class="label">STH update threshold (in seconds)</label>
                <div class="control">
                    <input
                        bind:value={settings.sth_update_threshold}
                        class="input"
                        type="number"
                        placeholder=""
                    />
                </div>
                <p class="help">
                    luCT will fetch a fresh STH for a log, if the existing one
                    is older than this value.
                </p>
            </div>

            <div class="field">
                <!-- svelte-ignore a11y-label-has-associated-control -->
                <label class="label">Report LRU cache size</label>
                <div class="control">
                    <input
                        bind:value={settings.report_lru_cache}
                        class="input"
                        type="number"
                        placeholder=""
                    />
                </div>
                <p class="help">
                    Larger number accelerates luCT's update speed but may
                    consume more RAM.
                </p>
            </div>

            <label class="checkbox">
                <input bind:checked={settings.debug_output} type="checkbox" />
                Debug output
            </label>

            <div class="field">
                <div class="control">
                    <button
                        on:click={store_and_reload}
                        class="button is-primary"
                        >Save settings and reload</button
                    >
                    <button on:click={export_store} class="button"
                        >Export store</button
                    >
                    <button on:click={import_store} class="button"
                        >Import store</button
                    >
                </div>
            </div>
        </div>
    </div>

    <div slot="footer">
        <p>
            <b class="card-footer-item"
                >Note: The settings are currently not being validated. If you
                set them wrong, luCT may stop working.
            </b>
        </p>
    </div>
</Page>

<style lang="sass">
input[type=number]::-webkit-inner-spin-button, 
input[type=number]::-webkit-outer-spin-button
    -webkit-appearance: none
    margin: 0

input[type=number] 
  -moz-appearance: textfield

</style>
