<script>
    let settings = JSON.parse(window.localStorage.getItem("settings"));
    let old_settings = settings;
    console.log(settings);

    function store_and_reload() {
        console.log(settings);
    }
</script>

<div class="card">
    <header class="card-header">
        <p class="card-header-title">Settings</p>
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

        <div class="field">
            <!-- svelte-ignore a11y-label-has-associated-control -->
            <label class="label">STH freshness threshold (in seconds)</label>
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
                luCT will fetch a fresh STH for a log, if the existing one is
                older than this value.
            </p>
        </div>

        <label class="checkbox">
            <input bind:checked={settings.debug_output} type="checkbox" />
            Debug output
        </label>

        <footer class="card-footer">
            <div class="control">
                <button
                    on:click={store_and_reload}
                    class="button is-primary card-footer-item"
                    >Save settings and reload</button
                >
            </div>
        </footer>
    </div>
</div>
