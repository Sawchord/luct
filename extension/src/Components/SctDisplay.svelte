<script>
    import Expandable from "./Expandable.svelte";
    import SthDisplay from "./SthDisplay.svelte";
    import TimeDisplay from "./TimeDisplay.svelte";

    export let sct;
</script>

<li>
    <Expandable>
        <span slot="name">
            <b>Log name:</b>
            {sct.log_name}
        </span>
        <ul>
            <li>
                <b>Validation time:</b>
                <TimeDisplay time={sct.signature_validation_time} />
            </li>
            <li>
                <SthDisplay
                    name="Inclusion proof"
                    ,
                    sth={sct.inclusion_proof}
                />
            </li>
            <li>
                <SthDisplay name="Latest STH" , sth={sct.latest_sth} />
            </li>
            <li>
                <b>Cached:</b>
                {sct.cached}
            </li>
        </ul>
    </Expandable>
</li>
