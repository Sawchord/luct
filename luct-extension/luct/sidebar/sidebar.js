import Report from "../components/report.js"
customElements.define('luct-report', Report);

let log = console.log.bind(console)

let windowId;
let tabId;
const contentBox = document.querySelector("#content_text");

browser.tabs.onActivated.addListener((tab) => {
    tabId = tab.tabId;
    update_content();

});

browser.runtime.onMessage.addListener((message) => {
    update_content();
});

browser.windows.getCurrent({ populate: true }).then(async (windowInfo) => {
    windowId = windowInfo.id;

    let tabs = await browser.tabs.query({ windowId, active: true });
    tabId = tabs[0].id;
    update_content();
});

async function update_content() {
    let report = await browser.runtime.sendMessage({ tabId })

    const content = document.querySelector("#content");
    const contentText = document.querySelector("#content_text");

    if (report) {
        let report_urls = Object.fromEntries(report.urls);

        content.replaceChildren();
        for (const [url, rep] of report.urls) {
            const reportElement = new Report(url, rep.report, rep.status);
            content.insertAdjacentElement("beforeend", reportElement);
        }

        contentText.textContent = "";
    } else {
        content.replaceChildren();
        contentText.textContent = "No data";
    }


}