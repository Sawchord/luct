<script>
    import Expandable from "./Expandable.svelte";
    import TimeDisplay from "./TimeDisplay.svelte";
    import UrlDisplay from "./UrlDisplay.svelte";

    //export let report;
</script>

<div class="card">
    <header class="card-header">
        <p class="card-header-title">"wikipedia.org"</p>
        <button class="card-header-icon" aria-label="more options">
            <span id="check-outer" class="icon">
                <i id="check-inner" class="fa" aria-hidden="true"></i>
            </span>
        </button>
    </header>
    <div class="card-content">
        <div class="content">
            <ul>
                <li><b> CA: </b> <span> Test </span></li>
                <li>
                    <Expandable>
                        <b slot="name"> Fingerprint</b>
                        <ul>
                            <li>aa:bb:cc:dd</li>
                        </ul>
                    </Expandable>
                </li>
                <li>
                    <b> Not valid before: </b>
                    <span>
                        <TimeDisplay time={new Date()} />
                    </span>
                </li>
                <li>
                    <b> Not valid after: </b>
                    <span>
                        <TimeDisplay time={new Date()} />
                    </span>
                </li>

                <li>
                    <Expandable>
                        <b slot="name"> Contains 2 scts</b>
                    </Expandable>
                </li>
                <li>
                    <UrlDisplay
                        urls={["wikipedia.org", "test.com", "luct.dev"]}
                    />
                </li>
            </ul>
        </div>
    </div>
</div>
