let log = console.log.bind(console)
let ALL_SITES = { urls: ['<all_urls>'] }
let extraInfoSpec = ['blocking'];

log(`Loading luCT extension`)


fetch(browser.runtime.getURL('assets/logs.toml'))
    .then(res => {
        res.text().then((logs) => log('parsed log'))
    })

function add_listener() {
    browser.webRequest.onHeadersReceived.addListener(async (details) => {
        log(`\n\nGot a request for ${details.url} with ID ${details.requestId}`)

        let requestId = details.requestId

        let securityInfo = await browser.webRequest.getSecurityInfo(requestId, {
            certificateChain: true,
            rawDER: true
        });

        log(`securityInfo: ${JSON.stringify(securityInfo, null, 2)}`)

    }, ALL_SITES, extraInfoSpec)

    log('Added listener')
}

add_listener()