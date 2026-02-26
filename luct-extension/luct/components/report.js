export default class Report extends HTMLElement {
    constructor(data) {
        super();

        this.urls = data.urls;
        this.report = data.report;

        const shadow = this.attachShadow({ mode: 'open' });
        const anchor = document.createElement('div');


        // FIXME: Don't use innerHTML
        let urlDisplay = `<b> Used by  ${this.urls.length} urls </b> <ul>`;
        for (const url of this.urls) {
            urlDisplay += `<li> ${url} </li>`
        }
        urlDisplay += "</ul>"

        anchor.innerHTML = `
            <link rel="stylesheet" href="https://maxcdn.bootstrapcdn.com/font-awesome/4.4.0/css/font-awesome.min.css">
            <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/bulma/1.0.4/css/bulma.min.css" integrity="sha512-yh2RE0wZCVZeysGiqTwDTO/dKelCbS9bP2L94UvOFtl/FKXcNAje3Y2oBg/ZMZ3LS1sicYk4dYVGtDex75fvvA==" crossorigin="anonymous" referrerpolicy="no-referrer" />
            
            <div class="card">
                <header class="card-header">
                    <p class="card-header-title">${this.report.cert_name}</p>
                    <button class="card-header-icon" aria-label="more options">
                        <span class="icon">
                            <i class="fa fa-angle-down" aria-hidden="true"></i>
                        </span>
                    </button>
                </header>
                <div class="card-content">
                    <div class="content">
                        <ul is="tree-view">
                            <li> <b> CA: </b>${this.report.ca_name}</li>
                            <li>
                                <b> Fingerprint </b> 
                                <ul> 
                                    <li> ${this.report.fingerprint} </li>
                                </ul> 
                            </li>
                            <li> <b>Not valid before: </b> <time is="date-time">${this.report.not_before}</time> </li>
                            <li> <b>Not valid after: </b> <time is="date-time">${this.report.not_after}</time> </li>
                            <li> 
                                <b> SCTs </b>
                                <ul>
                                    <li> SCT1 </li>
                                </ul>
                            </li>
                            <li> ${urlDisplay} </li>
                        </ul>
                    </div>
                </div>

            </div>
        `;

        shadow.appendChild(anchor);

    }

    connectedCallback() {
    }

    static define() {
        customElements.define("luct-report", Report);
    }
};
