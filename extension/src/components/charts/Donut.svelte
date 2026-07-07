<script>
    import Chart from "chart.js/auto";
    import { onMount } from "svelte/internal";

    export let data;
    export let onClick;
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
                        display: false,
                    },
                },
                onClick: (_event, elements) => {
                    if (elements.length > 0) {
                        const clickedElement = elements[0];
                        const dataIndex = clickedElement.index;
                        const sliceLabel = chart.data.labels[dataIndex];

                        if (onClick && typeof onClick === "function") {
                            onClick(sliceLabel);
                        }
                    }
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
