<script>
    import Donut from "../components/charts/Donut.svelte";
    import Page from "../components/Page.svelte";

    async function basic_statistics() {
        let stats = await browser.runtime.sendMessage("basic_statistics");
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
                        <div class="card-image">
                            <Donut data={stats.roots_cas} />
                        </div>

                        <footer class="card-footer">
                            <div class="content">
                                <b>Total: TODO: Implement</b>
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
                        <div class="card-image">
                            <Donut data={stats.scts} />
                        </div>
                        <footer class="card-footer">
                            <div class="content">
                                <b>Total: TODO: Implement</b>
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
    padding-left: 1.5rem;
    padding-right: 1.5rem;

.align-card-content
    min-height: 6rem;
</style>
