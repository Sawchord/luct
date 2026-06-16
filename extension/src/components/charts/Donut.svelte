<script>
    import Chart from "chart.js/auto";
    import { onMount } from "svelte/internal";

    export let data;
    let canvas;

    const labels = data.map((val) => val[0]);
    const values = data.map((val) => val[1]);

    onMount(() => {
        new Chart(canvas, {
            type: "doughnut",
            options: {
                cutout: "75%",
                plugins: {
                    legend: {
                        position: "bottom",
                    },
                },
            },
            data: {
                labels: labels,
                datasets: [
                    {
                        data: values,
                    },
                ],
            },
        });
    });
</script>

<canvas bind:this={canvas}></canvas>
