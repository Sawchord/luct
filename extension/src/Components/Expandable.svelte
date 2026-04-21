<script>
    export let name;
    let open = false;

    function click() {
        open = !open;
        console.log("Click" + open);
        return true;
    }
</script>

<li>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <span on:click={click}>
        <slot name="name"></slot>

        {#if open}
            <i class="fa fa-angle-down"></i>
        {:else}
            <i class="fa fa-angle-right"></i>
        {/if}
    </span>
    <span hidden={!open}>
        <slot></slot>
    </span>
</li>
