<div class="card">
    <div class="card-content">
        <div class="field">
            <!-- svelte-ignore a11y-label-has-associated-control -->
            <label class="label">Oblivious TLS proxy</label>
            <label class="checkbox">
                <input type="checkbox" />
                Use oblivious TLS proxy
            </label>
            <div class="control">
                <input class="input" type="text" placeholder="Url" />
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
                <input class="input" type="text" placeholder="" />
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
                <input class="input" type="text" placeholder="" />
            </div>
            <p class="help">
                luCT will fetch a fresh STH for a log, if the existing one is
                older than this value.
            </p>
        </div>

        <label class="checkbox">
            <input type="checkbox" />
            Debug output
        </label>

        <footer class="card-footer">
            <div class="control">
                <button class="button is-primary card-footer-item"
                    >Save settings and reload extension</button
                >
            </div>
        </footer>
    </div>
</div>
