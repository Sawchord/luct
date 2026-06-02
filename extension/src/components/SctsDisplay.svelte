<script>
    import Expandable from "./Expandable.svelte";
    import SctDisplay from "./SctDisplay.svelte";

    export let scts;
</script>

{#if scts}
    <Expandable>
        <b slot="name">Contains {scts.length} scts</b>
        <ul>
            {#each scts as sct}
                <li><SctDisplay {sct} /></li>
            {/each}
        </ul>
    </Expandable>
{/if}
