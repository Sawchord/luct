<script>
    import CertLifetime from "./CertLifetime.svelte";
    import Expandable from "./Expandable.svelte";
    import SctsDisplay from "./SctsDisplay.svelte";
    import StatusIcon from "./StatusIcon.svelte";
    import TimeDisplay from "./TimeDisplay.svelte";

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
                {#if report.report.error_description}
                    <div class="has-text-warning">
                        <b>Error: </b>
                        {report.report.error_description}
                    </div>
                {/if}

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
                    <CertLifetime
                        not_before={report.report.not_before}
                        not_after={report.report.not_after}
                    ></CertLifetime>

                    {#if report.report.scts}
                        <li>
                            <SctsDisplay scts={report.report.scts} />
                        </li>
                    {/if}
                </ul>
            </div>
        </div>
    </div>
{/if}
