<script>
    import Expandable from "./Expandable.svelte";
    import TimeDisplay from "./TimeDisplay.svelte";

    export let not_before;
    export let not_after;

    let lifetime = Math.round(
        (new Date(not_after) - new Date(not_before)) / (1000 * 60 * 60 * 24),
    );
</script>

<Expandable>
    <span slot="name">
        <b>TTL: </b>
        {lifetime} days
    </span>
    <ul>
        <li>
            <b> Not valid before: </b>
            <span>
                <TimeDisplay time={not_before} />
            </span>
        </li>
        <li>
            <b> Not valid after: </b>
            <span>
                <TimeDisplay time={not_after} />
            </span>
        </li>
    </ul>
</Expandable>
