export default class DateTime extends HTMLTimeElement {
    constructor() {
        super()

        this.datetime = new Date(this.innerText);
        this.innerText = this.datetime.toLocaleString()
    }

    static define() {
        customElements.define("date-time", DateTime, { extends: "time" });
    }
}