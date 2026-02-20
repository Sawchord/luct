import init, { Scanner } from './assets/wasm/luct_extension.js';

let log = console.log.bind(console)
let ALL_SITES = { urls: ['<all_urls>'] }
let extraInfoSpec = ['blocking'];

// TODO: Introduce in progress state
// TODO: Better management of tab security

class TabState {
    constructor() {
        this.tabs = new Map();
    }

    async updateTab(tabId, url, result) {
        if (tabId === -1) {
            // Calls to -1 are calls of the extension itself
            return;
        }

        var tab = this.tabs.get(tabId);
        if (!tab) {
            log("Initializing new tab: " + tabId);
            tab = new TabSecurity2(tabId, url);
        }

        tab.update_status(url, result);
        await tab.update_page_action();
        this.tabs.set(tabId, tab);
        //log(this);
    }


    async updateTabResult(tabId, url, result) {
        var tab = this.tabs.get(tabId);
        if (!tab) {
            log("Initializing new tab: " + tabId);
            tab = new TabSecurity(url);
        }

        tab.update(url, result);
        this.tabs.set(tabId, tab);
        await this.updateTabUrl(tabId);
    }

    async updateTabUrl(tabId) {
        let tab = this.tabs.get(tabId);
        if (!tab) {
            return;
        }

        if (tab.safety === "safe") {
            await browser.pageAction.setIcon({ tabId: tabId, path: "assets/icons/luct_safe.svg" })
            await browser.pageAction.show(tabId);
        } else {
            await browser.pageAction.setIcon({ tabId: tabId, path: "assets/icons/luct_unsafe.svg" })
            await browser.pageAction.show(tabId);
        }
    }

    deleteTab(tabId) {
        let toDelete = [];
        this.tabs.forEach((_value, key) => {
            if (key[0] === tabId) {
                toDelete.push(key);
            }
        });

        toDelete.forEach((key) => this.tabs.delete(key));
    }
}

class TabSecurity {
    constructor(url) {
        this.url = url;
        this.safety = "safe";
        this.checks = [];
    }

    update(url, result) {
        if (this.url !== url) {
            this.url = url;
            this.checks = [];
            this.safety = "safe"
        };

        this.checks.push(result);

        if (result.conclusion().is_unsafe()) {
            this.safety = "unsafe";
        } else if (this.safety !== "unsafe" && result.conclusion().is_inconclusive()) {
            this.safety = "inconclusive";
        }
    }
}


class TabSecurity2 {
    constructor(tabId, document_url) {
        this.tabId = tabId;
        this.document_url = document_url;
        this.urls = new Map();
    }

    update_status(url, status) {
        this.urls.set(url, status)
    }

    get_status() {
        var status = "safe";

        for (let [url, url_status] of this.urls) {
            if (!url_status) {
                status = null;
            } else if (url_status !== "safe") {
                status = url_status;
            }
        }

        return status;
    }

    async update_page_action() {
        log(this)
        if (this.get_status() === "safe") {
            await browser.pageAction.setIcon({ tabId: this.tabId, path: "assets/icons/luct_safe.svg" })
            await browser.pageAction.show(this.tabId);
        } else {
            await browser.pageAction.setIcon({ tabId: this.tabId, path: "assets/icons/luct_unsafe.svg" })
            await browser.pageAction.show(this.tabId);
        }
    }
}

log(`Loading luCT extension`)
var scanner;
var tabState = new TabState();

init().then(load_scanner).then(setup_tab_actions).then(add_listener)

function load_scanner() {
    fetch(browser.runtime.getURL('assets/log_list.json'))
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
        //log("Tab id: " + details.tabId)

        let requestId = details.requestId

        browser.webRequest.getSecurityInfo(requestId, {
            certificateChain: true,
            rawDER: true
        }).then(async (securityInfo) => {
            //log(details)
            //log(`securityInfo: ${JSON.stringify(securityInfo, null, 2)}`)
            let certs = securityInfo.certificates.map((info) => info.rawDER);

            tabState.updateTab(details.tabId, details.url, null);

            let report = await scanner.collect_report(details.url, certs);
            // Skip the recursive calls
            if (!(report)) {
                log("Skipping recursive request");
                return;
            }

            log(report);
            try {
                Scanner.evaluate_report(report);
                tabState.updateTab(details.tabId, details.url, "safe");
            } catch (error) {
                tabState.updateTab(details.tabId, details.url, error);
            }

            //log(certs)

            // let leads = scanner.collect_leads(details.url, certs);
            // //log(leads);
            // let investigations = leads.map((lead) => scanner.investigate_lead(lead).then((result) => {
            //     log("Investigated: " + lead.description());
            //     log("Conclusion: " + result.conclusion().description());
            //     return [lead, result]
            // }));

            // let results = await Promise.all(investigations);
            // results.forEach(async (result) => await tabState.updateTabResult(details.tabId, details.documentUrl, result[1]))
        });



    }, ALL_SITES, extraInfoSpec)

    log('Added listener')
}

function setup_tab_actions() {
    browser.tabs.onRemoved.addListener((tabId) => {
        log(`Tab ${tabId} was closed`)
        tabState.deleteTab(tabId);
    });

    browser.tabs.onUpdated.addListener(async (tabId, _changeInfo, tab) => {
        log(`Tab ${tabId} has updated url`)
        //log(tab)
        //await tabState.updateTabUrl(tabId);
        tabs.deleteTab(tabId);

    },
        { properties: ["url"] }
    )

}

