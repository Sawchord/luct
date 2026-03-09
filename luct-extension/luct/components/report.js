export default class Report extends HTMLElement {
    constructor(data) {
        super();

        this.urls = data.urls;
        this.report = data.report;

        const shadow = this.attachShadow({ mode: 'open' });
        this.shadow = shadow;

        const anchor = document.createElement('div');


        // FIXME: Don't use innerHTML, use templates and generate the inner data as elements
        const sthDisplay = (name, sth) => {
            return `
            <b> ${name}: ${sth.height} </b>
            <ul>
                <li> <b> Timestamp: </b> <time is="date-time">${sth.timestamp}</time> </li>
                <li> <b> Verification time: </b> <time is="date-time">${sth.verification_time}</time> </li>
            </ul>`
        }

        const sctDisplay = (sct) => {
            if (sct.error_description) {
                return `
                    <b> Log: ${sct.log_name} </b>
                        <ul>
                        <li> <b> Error </b> ${sct.error_description} </li> 
                    </ul>
                `
            }

            return `
            <b> Log: ${sct.log_name} </b> 
            <ul>
                <li> <b> Validation time: </b> <time is="date-time">${sct.signature_validation_time}</time> </li>
                <li> ${sthDisplay("Inclusion proof", sct.inclusion_proof)} </li>
                <li> ${sthDisplay("Latest STH", sct.latest_sth)} </li>
                <li> <b> Cached: </b> ${sct.cached} </li>
            </ul>`
        }

        const sctsDisplay = () => {
            let display = `<b> Constains  ${this.report.scts.length} scts </b> <ul>`;
            for (const sct of this.report.scts) {
                display += `<li> ${sctDisplay(sct)} </li>`;
            }

            display += "</ul>";
            return display
        }


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
                        <ul is="tree-view" id="tree-view">
                            <li> <b> CA: </b>${this.report.ca_name}</li>
                            <li>
                                <b> Fingerprint </b> 
                                <ul> 
                                    <li> ${this.report.fingerprint} </li>
                                </ul> 
                            </li>
                            <li> <b>Not valid before: </b> <time is="date-time">${this.report.not_before}</time> </li>
                            <li> <b>Not valid after: </b> <time is="date-time">${this.report.not_after}</time> </li>
                            <li> ${sctsDisplay()} </li>
                            <li id="url"> </li>
                        </ul>
                    </div>
                </div>
            </div>
        `;

        shadow.appendChild(anchor);

    }

    urlDisplay() {
        const anchor = this.shadow.getElementById("url");

        const summary = document.createElement("b");
        summary.innerText = ` Used by  ${this.urls.length} urls`;
        anchor.appendChild(summary);


        const ul = document.createElement("ul");
        for (const url of this.urls) {
            const li = document.createElement("li");
            li.innerText = url;
            ul.appendChild(li);
        }
        anchor.appendChild(ul)
    }

    connectedCallback() {
        this.urlDisplay();
    }



    static define() {
        customElements.define("luct-report", Report);
    }
};
