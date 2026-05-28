<script>
    export let status;

    var icon = "";
    var color = "";

    switch (status) {
        case "processing":
            icon = "fa-refresh";
            color = "";
            break;
        case "safe":
            icon = "fa-check";
            color = "has-text-success";
            break;
        case "warn":
            icon = "fa-warning";
            color = "has-text-warning";
            break;
        case "unsafe":
            icon = "fa-times";
            color = "has-text-danger";
            break;
        default:
            icon = "";
            color = "";
    }
    icon = `fa ${icon}`;
    color = `icon ${color}`;
</script>

<button class="card-header-icon">
    <span class={color}>
        <i class={icon} aria-hidden="true"></i>
    </span>
</button>
