<script>
    import Donut from "../components/charts/Donut.svelte";
    import Page from "../components/Page.svelte";

    let data = [
        ["A", 200],
        ["B", 100],
        ["C", 50],
    ];
</script>

<Page>
    <div slot="content">
        <b>TODO: Implement the dashboard page</b>

        <div class="columns">
            <div class="column">
                <div class="card">
                    <header class="card-header">
                        <p class="card-header-title">Root CAs</p>
                    </header>
                    <div class="card-image">
                        <Donut {data} />
                    </div>
                    <footer class="card-footer">
                        <div class="content">
                            <b>Total: 350</b>
                        </div>
                    </footer>
                </div>
            </div>
            <div class="column">
                <div class="card">
                    <header class="card-header">
                        <p class="card-header-title">Root CAs</p>
                    </header>
                    <div class="card-image">
                        <Donut {data} />
                    </div>
                    <footer class="card-footer">
                        <div class="content">
                            <b>Total: 350</b>
                        </div>
                    </footer>
                </div>
            </div>
        </div>
    </div>
</Page>
