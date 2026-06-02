<script>
    export let status;

    function getIcon(status) {
        switch (status) {
            case "processing":
                return "fa fa-refresh";
            case "safe":
                return "fa fa-check";
            case "warn":
                return "fa fa-warning";
            case "unsafe":
                return "fa fa-times";
            default:
                return "fa";
        }
    }

    function getColor(status) {
        switch (status) {
            case "processing":
                return "icon";
            case "safe":
                return "icon has-text-success";
            case "warn":
                return "icon has-text-warning";
            case "unsafe":
                return "icon has-text-danger";
            default:
                return "icon";
        }
    }
</script>

<button class="card-header-icon">
    {#key status}
        <span class={getColor(status)}>
            <i class={getIcon(status)} aria-hidden="true"></i>
        </span>
    {/key}
</button>
