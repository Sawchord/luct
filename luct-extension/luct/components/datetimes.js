export default class DateTime extends HTMLElement {
    constructor() {
        super()

        const shadow = this.attachShadow({ mode: 'open' });
        const anchor = document.createElement('time');
        anchor.setAttribute("id", "time-display");

        shadow.appendChild(anchor);
    }

    connectedCallback() {
        this.datetime = new Date(this.getHTML());

        const anchor = this.shadowRoot.querySelector("#time-display");
        anchor.setAttribute("datetime", this.datetime.toLocaleString());
        anchor.innerText = this.datetime.toLocaleString();

    }
}