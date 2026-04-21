<script>
    import Expandable from "./Expandable.svelte";
    import TimeDisplay from "./TimeDisplay.svelte";

    export let name;
    export let sth;
</script>

<Expandable>
    <span slot="name"><b>{name}: </b>{sth.height}</span>
    <ul>
        <li><b>Timestamp: </b><TimeDisplay time={sth.timestamp} /></li>
        <li>
            <b>Verification time: </b><TimeDisplay
                time={sth.verification_time}
            />
        </li>
    </ul>
</Expandable>
