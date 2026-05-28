<script>
    export let status;

    var icon = "";
    var color = "";

    switch (status) {
        case "safe":
            icon = "fa-check";
            color = "has-text-success";
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
