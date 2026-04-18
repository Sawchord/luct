export default class DateTime extends HTMLTimeElement {
    constructor(datetime) {
        super()

        if (datetime) {
            this.datetime = new Date(datetime);
        } else {
            this.datetime = new Date(this.innerText);
        }

        this.innerText = this.datetime.toLocaleString()
    }

    connectedCallback() {
    }

    static define() {
        customElements.define("date-time", DateTime, { extends: "time" });
    }
}