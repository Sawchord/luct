export default class TreeView extends HTMLUListElement {
    constructor() {
        super();

        console.log("Hi");

        const uls = this.querySelectorAll("ul");
        for (const ul of uls) {
            ul.style.display = "none";
        }


        const lis = this.querySelectorAll("li");
        for (const li of lis) {

            const ul = li.querySelector("ul");
            if (ul) {
                const headline = li.firstChild;

                const newImage = document.createElement("i");
                newImage.setAttribute("class", "fa fa-angle-right")

                const newSpan = document.createElement("span");
                newSpan.appendChild(newImage);
                newSpan.append(headline);

                li.insertBefore(newSpan, ul);

                newSpan.addEventListener("click", (event) => {
                    if (ul.style.display === "none") {
                        ul.style.display = "block";
                        newImage.setAttribute("class", "fa fa-angle-down")
                    } else {
                        ul.style.display = "none";
                        newImage.setAttribute("class", "fa fa-angle-right")
                    }
                });
            }
        }

    }

    static define() {
        customElements.define("tree-view", TreeView, { extends: "ul" })
    }
}