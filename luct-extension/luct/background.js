var log = console.log.bind(console)

log(`\n\nTLS browser extension loaded`)

var ALL_SITES = { urls: ['<all_urls>'] }
var extraInfoSpec = ['blocking'];

browser.webRequest.onHeadersReceived.addListener(async function (details) {
    log(`\n\nGot a request for ${details.url} with ID ${details.requestId}`)

    var requestId = details.requestId

    var securityInfo = await browser.webRequest.getSecurityInfo(requestId, {
        certificateChain: true,
        rawDER: false
    });

    log(`securityInfo: ${JSON.stringify(securityInfo, null, 2)}`)

}, ALL_SITES, extraInfoSpec)

log('Added listener')

