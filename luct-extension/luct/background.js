import init, { Scanner } from './assets/wasm/luct_extension.js';

let log = console.log.bind(console)
let ALL_SITES = { urls: ['<all_urls>'] }
let extraInfoSpec = ['blocking'];

let activeTab = -1;

// TODO: Introduce in progress state
// TODO: Better management of tab security

class TabState {
    constructor() {
        this.tabs = new Map();
    }

    async updateTab(tabId, url, report, result) {
        if (tabId === -1) {
            // Calls to -1 are calls of the extension itself
            return;
        }

        var tab = this.tabs.get(tabId);
        if (!tab) {
            log("Initializing new tab: " + tabId);
            tab = new TabSecurity(tabId, url);
        }

        tab.update_status(url, report, result);
        await tab.update_page_action();
        this.tabs.set(tabId, tab);
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

let scanner;
let tabState = new TabState();

class TabSecurity {
    constructor(tabId, document_url) {
        this.tabId = tabId;
        this.document_url = document_url;
        this.urls = new Map();
    }

    async update_status(url, report, status) {
        this.urls.set(url, { report, status })

        if (this.tabId === activeTab && await browser.sidebarAction.isOpen({})) {
            browser.runtime.sendMessage(this)
        }
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
        let requestId = details.requestId

        browser.webRequest.getSecurityInfo(requestId, {
            certificateChain: true,
            rawDER: true
        }).then(async (securityInfo) => {
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
                tabState.updateTab(details.tabId, details.url, report, "safe");
            } catch (error) {
                tabState.updateTab(details.tabId, details.url, report, error);
            }
        });



    }, ALL_SITES, extraInfoSpec);

    browser.runtime.onMessage.addListener((message, _sender, respond) => {
        let tabData = tabState.tabs.get(message.tabId);
        activeTab = message.tabId;
        respond(tabData);
    });

    log('Added listeners')
}

function setup_tab_actions() {
    browser.tabs.onRemoved.addListener((tabId) => {
        log(`Tab ${tabId} was closed`)
        tabState.deleteTab(tabId);
    });

    browser.tabs.onUpdated.addListener(async (tabId, _changeInfo, tab) => {
        log(`Tab ${tabId} has updated url`)
        tabs.deleteTab(tabId);
    },
        { properties: ["url"] }
    );

}

