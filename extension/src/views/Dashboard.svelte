<script>
    import Donut from "../components/charts/Donut.svelte";
    import Page from "../components/Page.svelte";

    async function basic_statistics() {
        let stats = await browser.runtime.sendMessage("basic_statistics");

        stats.roots_cas_total = stats.roots_cas.reduce(
            (total, root_ca) => total + root_ca[2],
            0,
        );

        stats.scts_total = stats.scts.reduce((total, sct) => total + sct[1], 0);

        console.log(stats);
        return stats;
    }
</script>

<Page>
    <div slot="content">
        {#await basic_statistics()}
            <b>Loading statistics</b>
        {:then stats}
            <div class="columns column-spacing">
                <div class="column">
                    <div class="card">
                        <header class="card-header">
                            <p class="card-header-title">Root CAs</p>
                        </header>
                        <div class="card-content align-card-content">
                            The number of scanned certificates, partitioned by
                            their root certificate authority.
                        </div>
                        <div class="card-image card-spacing">
                            <Donut
                                labels={stats.roots_cas.map((val) => val[0])}
                                values={stats.roots_cas.map((val) => val[2])}
                                onClick={(label) => {
                                    const fp = stats.roots_cas.find(
                                        (el) => el[0] === label,
                                    );

                                    if (fp) {
                                        // TODO: Navigate to RootCA overview of clicked log
                                        console.log(
                                            "Clicked RootCA: " +
                                                label +
                                                " Fingerprint: " +
                                                fp[1],
                                        );
                                    }
                                }}
                            />
                        </div>

                        <footer class="card-footer total-spacing">
                            <div class="content">
                                <b>Total: {stats.roots_cas_total}</b>
                            </div>
                        </footer>
                    </div>
                </div>
                <div class="column">
                    <div class="card">
                        <header class="card-header">
                            <p class="card-header-title">Scanned SCTs</p>
                        </header>
                        <div class="card-content align-card-content">
                            The number of validated SCTs, partitioned by their
                            logs.
                        </div>
                        <div class="card-image card-spacing">
                            <Donut
                                labels={stats.scts.map((val) => val[0])}
                                values={stats.scts.map((val) => val[1])}
                                onClick={(label) =>
                                    // TODO: Navigate to the log overview of clicked log
                                    console.log(
                                        "Clicked Scanned SCT: " + label,
                                    )}
                            />
                        </div>
                        <footer class="card-footer total-spacing">
                            <div class="content">
                                <b>Total: {stats.scts_total}</b>
                            </div>
                        </footer>
                    </div>
                </div>
            </div>
        {/await}
    </div>
</Page>

<style lang="sass">
.column-spacing
    padding-left: 0.5rem;
    padding-right: 0.5rem;

.align-card-content
    min-height: 6rem;

.card-spacing
    padding-left: 0.5rem;
    padding-right: 0.5rem;
    padding-bottom: 0.5rem;

.total-spacing
    padding: 0.5rem;
</style>
