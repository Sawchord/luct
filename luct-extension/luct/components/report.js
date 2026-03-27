


function li(inner) {
    const li = document.createElement("li");
    li.appendChild(inner);
    return li;
}

export default class Report extends HTMLElement {
    constructor(data) {
        super();

        this.urls = data.urls;
        this.report = data.report;

        const shadow = this.attachShadow({ mode: 'open' });
        this.shadow = shadow;

        const anchor = document.createElement('div');

        anchor.innerHTML = `
            <link rel="stylesheet" href="https://maxcdn.bootstrapcdn.com/font-awesome/4.7.0/css/font-awesome.min.css">
            <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/bulma/1.0.4/css/bulma.min.css" integrity="sha512-yh2RE0wZCVZeysGiqTwDTO/dKelCbS9bP2L94UvOFtl/FKXcNAje3Y2oBg/ZMZ3LS1sicYk4dYVGtDex75fvvA==" crossorigin="anonymous" referrerpolicy="no-referrer" />
            
            <div class="card">
                <header class="card-header">
                    <p class="card-header-title" id="cert-name"></p>
                    <button class="card-header-icon" aria-label="more options">
                        <span class="icon">
                            <i class="fa fa-angle-down" aria-hidden="true"></i>
                        </span>
                    </button>
                </header>
                <div class="card-content">
                    <div class="content">
                        <ul is="tree-view" id="tree-view">
                            <li> <b> CA: </b> <span id="ca-name"></span> </li>
                            <li>
                                <b> Fingerprint </b> 
                                <ul> 
                                    <li id="fingerprint"></li>
                                </ul> 
                            </li>
                            <li id="not-before"></li>
                            <li id="not-after"></li>
                            <li id="scts"> </li>
                            <li id="url"> </li>
                        </ul>
                    </div>
                </div>
            </div>
        `;

        shadow.appendChild(anchor);

    }

    valueDisplay(name, value) {
        const anchor = document.createElement("span");

        const b = document.createElement("b");
        b.innerText = name;
        anchor.appendChild(b);

        const val = document.createElement("time");
        val.innerText = value;
        anchor.appendChild(val);

        return anchor
    }

    timeDisplay(name, time) {
        return this.valueDisplay(name, new Date(time).toLocaleString())
    }


    sthDisplay(sth) {
        const ul = document.createElement("ul");
        ul.appendChild(li(this.timeDisplay("Timestamp: ", sth.timestamp)));
        ul.appendChild(li(this.timeDisplay("Verification time: ", sth.verification_time)));
        return ul;
    }

    sctDisplay(sct) {
        const ul = document.createElement("ul");

        if (sct.error_description) {
            ul.appendChild(li(this.valueDisplay("Error: ", sct.error_description)));
        } else {
            ul.appendChild(li(this.timeDisplay("Validation time: ", sct.signature_validation_time)));

            const inclusionProof = li(this.valueDisplay(" Inclusion proof: ", sct.inclusion_proof.height));
            inclusionProof.appendChild(this.sthDisplay(sct.inclusion_proof));
            ul.appendChild(inclusionProof);

            const latestSth = li(this.valueDisplay(" Latest STH: ", sct.latest_sth.height));
            latestSth.appendChild(this.sthDisplay(sct.latest_sth));
            ul.appendChild(latestSth);

            ul.appendChild(li(this.valueDisplay("Cached: ", sct.cached)));

        }

        return ul;
    }

    sctsDisplay() {
        const anchor = this.shadow.getElementById("scts");

        const summary = document.createElement("b");
        summary.innerText = ` Contains ${this.report.scts.length} scts`;
        anchor.appendChild(summary);

        const ul = document.createElement("ul");

        for (const sct of this.report.scts) {
            const l = li(this.valueDisplay(" Log name: ", sct.log_name));
            l.appendChild(this.sctDisplay(sct));
            ul.appendChild(l);
        }
        anchor.appendChild(ul)

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
        this.shadow.getElementById("cert-name").innerText = this.report.cert_name;
        this.shadow.getElementById("ca-name").innerText = this.report.ca_name;
        this.shadow.getElementById("fingerprint").innerText = this.report.fingerprint;

        this.shadow.getElementById("not-before").appendChild(this.timeDisplay("Not valid before: ", this.report.not_before));
        this.shadow.getElementById("not-after").appendChild(this.timeDisplay("Not valid after: ", this.report.not_after));

        this.sctsDisplay();
        this.urlDisplay();
    }



    static define() {
        customElements.define("luct-report", Report);
    }
};
