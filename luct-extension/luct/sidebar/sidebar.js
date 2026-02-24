let log = console.log.bind(console)

let windowId;
let tabId;
const contentBox = document.querySelector("#content_text");

browser.tabs.onActivated.addListener((tab) => {
    tabId = tab.tabId;
    update_content();

});

browser.windows.getCurrent({ populate: true }).then(async (windowInfo) => {
    log(windowInfo)
    windowId = windowInfo.id;

    let tabs = await browser.tabs.query({ windowId, active: true });
    tabId = tabs[0].id;
    update_content();
});

async function update_content() {
    let report = await browser.runtime.sendMessage({ tabId })

    if (report) {
        let report_urls = Object.fromEntries(report.urls);
        document.querySelector("#content_text").textContent = JSON.stringify(report) + JSON.stringify(report_urls);
    } else {
        document.querySelector("#content_text").textContent = "No data";
    }


}