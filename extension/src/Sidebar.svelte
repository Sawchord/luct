<script>
    import Expandable from "./Components/Expandable.svelte";
    import Report from "./Components/Report.svelte";

    async function get_testdata() {
        let data = await fetch("../testdata/test-report.json");
        console.log(data);
        let json = await data.json();
        console.log(json);
        return json;
    }
</script>

<div>TODO: Implement the sidebar component</div>

<Expandable name="Test">
    <Expandable name="Subitem"></Expandable>
</Expandable>

{#await get_testdata()}
    <p>Loading</p>
{:then report}
    <Report {report}></Report>
{/await}
