<div class="page-container">
    <div class="content-slot">
        <slot name="content"></slot>
    </div>

    <footer class="footer footer-slot">
        <div class="content has-text-centered">
            <slot name="footer"></slot>
        </div>
    </footer>
</div>

<style lang="sass">
$footer-height: 6rem

.page-container
    position: relative;
    min-height: 100vh;

.content-slot
    padding-bottom: $footer-height

.footer-slot 
    position: fixed
    left: 0
    bottom: 0
    width: 100%
    text-align: center
    padding: 1rem
    height: $footer-height;
</style>
