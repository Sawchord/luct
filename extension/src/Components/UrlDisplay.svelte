<script>
    import Expandable from "./Expandable.svelte";

    export let urls;
</script>

<Expandable>
    <b slot="name"> Used by {urls.length} domains</b>
    <ul>
        {#each urls as url}
            <li>{url}</li>
        {/each}
    </ul>
</Expandable>
