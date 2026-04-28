<script>
    import Expandable from "./Expandable.svelte";
    import SthDisplay from "./SthDisplay.svelte";
    import TimeDisplay from "./TimeDisplay.svelte";

    export let sct;
</script>

<li>
    <Expandable>
        <span slot="name">
            {#if sct.log_name}
                <b>Log name:</b>
                {sct.log_name}
            {:else}
                <b>Unrecognized log</b>
            {/if}
        </span>

        <ul>
            <li>
                <Expandable>
                    <b slot="name">Log id</b>
                    <ul>
                        <li>{sct.log_id}</li>
                    </ul>
                </Expandable>
            </li>
            {#if sct.signature_validation_time}
                <li>
                    <b>Validation time:</b>
                    <TimeDisplay time={sct.signature_validation_time} />
                </li>
            {/if}
            {#if sct.index}
                <li>
                    <b>Index:</b>
                    {sct.index}
                </li>
            {/if}
            {#if sct.inclusion_proof}
                <li>
                    <SthDisplay
                        name="Inclusion proof"
                        ,
                        sth={sct.inclusion_proof}
                    />
                </li>
            {/if}
            {#if sct.latest_sth}
                <li>
                    <SthDisplay name="Latest STH" , sth={sct.latest_sth} />
                </li>
            {/if}
            {#if sct.error_description}
                <li>
                    <b>Error:</b>
                    {sct.error_description}
                </li>
            {/if}
        </ul>
    </Expandable>
</li>
