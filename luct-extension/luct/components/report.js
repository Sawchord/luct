export default class Report extends HTMLElement {
    constructor(url, report, status) {
        super();

        this.url = url;
        this.report = report;
        this.status = status;

        const shadow = this.attachShadow({ mode: 'open' });

        const anchor = document.createElement('div');

        anchor.innerHTML = `
            <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@1.0.4/css/bulma.min.css" />
            <div class="card">
                <header class="card-header">
                    <p class="card-header-title">${this.url}</p>
                    <button class="card-header-icon" aria-label="more options">
                        <span class="icon">
                            <i class="fas fa-angle-down" aria-hidden="true"></i>
                        </span>
                    </button>
                </header>
                <div class="card-content">
                    <div class="content">
                        ${JSON.stringify(this.report)}
                    </div>
                </div>
                <footer class="card-footer">
                    <a href="#" class="card-footer-item">Save</a>
                    <a href="#" class="card-footer-item">Edit</a>
                    <a href="#" class="card-footer-item">Delete</a>
                </footer>
            </div>
        `;

        shadow.appendChild(anchor);

    }

    connectedCallback() {
        console.log("connectedCallback")
    }
};