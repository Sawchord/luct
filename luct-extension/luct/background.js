import init, { Scanner } from './assets/wasm/luct_extension.js';

let log = console.log.bind(console)
let ALL_SITES = { urls: ['<all_urls>'] }
let extraInfoSpec = ['blocking'];

log(`Loading luCT extension`)
var scanner;
var tabState = new Map();

init().then(load_scanner).then(add_listener).then(setup_tab_actions)

function load_scanner() {
    fetch(browser.runtime.getURL('assets/logs.toml'))
        .then(res => {
            res.text().then((logs) => {
                log('parsed log');
                scanner = new Scanner(logs);
            })
        })
}

function add_listener() {
    browser.webRequest.onHeadersReceived.addListener(async (details) => {
        log(`Got a request for ${details.url} with ID ${details.requestId}`)
        log("Tab id: " + details.tabId)

        let requestId = details.requestId

        let securityInfo = await browser.webRequest.getSecurityInfo(requestId, {
            certificateChain: true,
            rawDER: true
        });

        //log(`securityInfo: ${JSON.stringify(securityInfo, null, 2)}`)
        let certs = securityInfo.certificates.map((info) => info.rawDER);
        //log(certs)

        let leads = scanner.collect_leads(details.url, certs);
        //log(leads);
        let investigations = leads.map((lead) => scanner.investigate_lead(lead).then((result) => {
            log("Investigated: " + lead.description());
            log("Conclusion: " + result.conclusion().description());
            return [lead, result]
        }));

        let results = await Promise.all(investigations);

        if (results.find((result) => result[1].conclusion().is_unsafe())) {
            log("WEBSITE UNSAFE");
        } else if (results.find((result) => result[1].conclusion().is_inconclusive())) {
            log("Cannot determine safety of website");
        } else {
            log("The website is safe");
        }

        // TODO: We need to check the state of the tab, and potentially
        // downgrade the security if we find inconclusive of unsafe results
        // then update the pageaction

    }, ALL_SITES, extraInfoSpec)

    log('Added listener')
}

function setup_tab_actions() {
    browser.tabs.onRemoved.addListener((tabId) => {
        log(`Tab ${tabId} was closed`)
        // TODO: Remove the tabState corresponding to this tab
    });

    browser.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
        log(`Tab ${tabId} has updated url`)
        log(tab)
        // TODO: Remove the tabState and page action
        // NOTE: The initial calls to the new url happen before this event is
        // triggered. We need to map tabId -> url -> state, and then remove
        // all url -> state mappings here that don't belong to the current url
    },
        { properties: ["url"] }
    )

}