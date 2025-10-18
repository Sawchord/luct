import init, { Scanner } from './assets/wasm/luct_extension.js';

let log = console.log.bind(console)
let ALL_SITES = { urls: ['<all_urls>'] }
let extraInfoSpec = ['blocking'];

log(`Loading luCT extension`)
var scanner;

init().then(load_scanner).then(add_listener)

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
        //log(details)


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
        //log(results)

    }, ALL_SITES, extraInfoSpec)

    log('Added listener')
}

