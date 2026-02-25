export default class Report extends HTMLElement {
    constructor(url, report, status) {
        super();

        this.url = url;
        this.report = report;
        this.status = status;

        const shadow = this.attachShadow({ mode: 'open' });
        const anchor = document.createElement('div');

        anchor.innerHTML = `
            <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/7.0.1/css/all.min.css" integrity="sha512-2SwdPD6INVrV/lHTZbO2nodKhrnDdJK9/kg2XD1r9uGqPo1cUbujc+IYdlYdEErWNu69gVcYgdxlmVmzTWnetw==" crossorigin="anonymous" referrerpolicy="no-referrer" />    
            <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/bulma/1.0.4/css/bulma.min.css" integrity="sha512-yh2RE0wZCVZeysGiqTwDTO/dKelCbS9bP2L94UvOFtl/FKXcNAje3Y2oBg/ZMZ3LS1sicYk4dYVGtDex75fvvA==" crossorigin="anonymous" referrerpolicy="no-referrer" />
            
            <div class="card">
                <header class="card-header">
                    <p class="card-header-title">${this.url}</p>
                    <button class="card-header-icon" aria-label="more options">
                        <span class="icon">
                            <i class="fa fa-angle-down" aria-hidden="true"></i>
                        </span>
                    </button>
                </header>
                <div class="card-content">
                    <div class="content">
                        <div>
                            <b>CA: </b>
                            ${this.report.ca_name}
                        </div>
                        <div>
                            <b>Not valid before: </b> 
                            <date-time>${this.report.not_before}</date-time>
                        </div>
                        <div>
                            <b>Not valid after: </b> 
                            <date-time>${this.report.not_after}</date-time>
                        </div>
                        ${JSON.stringify(this.report)}
                    </div>
                </div>
            </div>
        `;

        shadow.appendChild(anchor);

    }

    connectedCallback() {
    }
};
