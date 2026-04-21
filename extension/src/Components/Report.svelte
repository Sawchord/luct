<script>
    import Expandable from "./Expandable.svelte";
    import SctsDisplay from "./SctsDisplay.svelte";
    import TimeDisplay from "./TimeDisplay.svelte";
    import UrlDisplay from "./UrlDisplay.svelte";

    export let report;

    var icon = "";
    if (report && report.status === "safe") {
        icon = "fa fa-check";
    }
</script>

{#if report}
    <div class="card">
        <header class="card-header">
            <p class="card-header-title">{report.report.cert_subject}</p>
            <button class="card-header-icon" aria-label="more options">
                <span class="icon has-text-success">
                    <i class={icon} aria-hidden="true"></i>
                </span>
            </button>
        </header>
        <div class="card-content">
            <div class="content">
                <ul>
                    <li><b> CA: </b> <span>{report.report.ca_issuer}</span></li>
                    <li>
                        <Expandable>
                            <b slot="name"> Fingerprint</b>
                            <ul>
                                <li>{report.report.fingerprint}</li>
                            </ul>
                        </Expandable>
                    </li>
                    <li>
                        <b> Not valid before: </b>
                        <span>
                            <TimeDisplay time={report.report.not_before} />
                        </span>
                    </li>
                    <li>
                        <b> Not valid after: </b>
                        <span>
                            <TimeDisplay time={report.report.not_after} />
                        </span>
                    </li>

                    <li>
                        <SctsDisplay scts={report.report.scts} />
                    </li>
                    <li>
                        <UrlDisplay urls={report.urls} />
                    </li>
                </ul>
            </div>
        </div>
    </div>
{/if}
