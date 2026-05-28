<script>
    import Expandable from "./Expandable.svelte";
    import SctsDisplay from "./SctsDisplay.svelte";
    import StatusIcon from "./StatusIcon.svelte";
    import TimeDisplay from "./TimeDisplay.svelte";
    // import UrlDisplay from "./UrlDisplay.svelte";

    export let report;
</script>

{#if report}
    <div class="card">
        <header class="card-header">
            <p class="card-header-title">{report.report.cert_subject}</p>
            <StatusIcon status={report.status}></StatusIcon>
        </header>
        <div class="card-content">
            <div class="content">
                <ul>
                    <li><b> CA: </b> <span>{report.report.ca_issuer}</span></li>
                    <li>
                        <Expandable>
                            <b slot="name">Fingerprint</b>
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
                    <!-- 
                        NOTE: Displaying URLs can be overwhelming and processing them takes a lot of resources
                        TODO: Remove URL reporting to sidebar altogether
                    -->
                    <!-- <li>
                        <UrlDisplay urls={report.urls} />
                    </li> -->
                    {#if report.report.error_description}
                        <li>
                            <b>Error: </b>
                            {report.report.error_description}
                        </li>
                    {/if}
                </ul>
            </div>
        </div>
    </div>
{/if}
