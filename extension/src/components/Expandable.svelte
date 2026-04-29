<script>
    let open = false;

    function click() {
        open = !open;
        return true;
    }
</script>

<li>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <span on:click={click}>
        {#if open}
            <i class="fa fa-angle-down"></i>
        {:else}
            <i class="fa fa-angle-right"></i>
        {/if}

        <slot name="name"></slot>
    </span>
    <span hidden={!open}>
        <slot></slot>
    </span>
</li>
