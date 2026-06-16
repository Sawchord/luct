<script>
    import Chart from "chart.js/auto";
    import { onMount } from "svelte/internal";

    export let data;
    let canvas;
    let chart;

    const labels = data.map((val) => val[0]);
    const values = data.map((val) => val[1]);

    function drawChart() {
        chart = new Chart(canvas, {
            type: "doughnut",
            options: {
                cutout: "60%",
                plugins: {
                    legend: {
                        //position: "bottom",
                        display: false,
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
    }

    window.addEventListener("resize", () => {
        chart.destroy();
        drawChart();
    });

    onMount(drawChart);
</script>

<canvas bind:this={canvas}></canvas>
