import Report from "../components/report.js"
import DateTime from "../components/datetimes.js"
import TreeView from "../components/treeview.js";


Report.define();
DateTime.define();
TreeView.define();

let log = console.log.bind(console)

let windowId;
let tabId;

browser.tabs.onActivated.addListener((tab) => {
    tabId = tab.tabId;
    update_content();

});

browser.runtime.onMessage.addListener((_message) => {
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

    const content = document.getElementById("content");

    if (report) {
        let certs = new Map();

        for (const [url, rep] of report.urls) {
            if (!rep.report) {
                continue;
            }

            let fingerprint = rep.report.fingerprint
            let existing_entry = certs.get(fingerprint);
            if (existing_entry) {
                existing_entry.urls.push(url);
                if (existing_entry.status === 'safe' && rep.status !== 'safe') {
                    existing_entry.status = rep.status;
                }
                certs.set(fingerprint, existing_entry);
            } else {
                certs.set(fingerprint, { report: rep.report, urls: [] });
            }
        }

        content.replaceChildren();
        for (const [_fp, data] of certs) {
            const reportElement = new Report(data);
            console.log(data);
            content.insertAdjacentElement("beforeend", reportElement);
        }

    } else {
        content.replaceChildren();
    }


}