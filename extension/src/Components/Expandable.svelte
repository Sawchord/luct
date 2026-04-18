<script>
    export let name;
    let open = false;

    function click() {
        open = !open;
        console.log("Click" + open);
        return true;
    }
</script>

<ul>
    <li>
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <span on:click={click}>
            {name}
            {open}
            {#if open}
                <i class="fa fa-angle-down"></i>
            {:else}
                <i class="fa fa-angle-right"></i>
            {/if}
        </span>
        <slot></slot>
    </li>
</ul>
