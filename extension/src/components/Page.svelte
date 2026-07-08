<div class="page-container">
    {#if $$slots.header}
        <header class="header header-slot">
            <slot name="header"></slot>
        </header>
    {/if}

    {#if $$slots.footer}
        <div class="content-slot">
            <slot name="content"></slot>
        </div>
        <footer class="footer footer-slot">
            <div class="content has-text-centered">
                <slot name="footer"></slot>
            </div>
        </footer>
    {:else}
        <div class="">
            <slot name="content"></slot>
        </div>
    {/if}
</div>

<style lang="sass">
$footer-height: 3rem

.header-slot
    position: sticky
    top: 0
    text-align: center
    align-content: center
    z-index: 1000

.page-container
    position: relative
    min-height: 100vh

.content-slot
    padding-bottom: $footer-height

.footer-slot 
    position: fixed
    left: 0
    bottom: 0
    width: 100%
    display: flex
    align-items: center
    justify-content: center
    padding: 1rem
    height: $footer-height
</style>
