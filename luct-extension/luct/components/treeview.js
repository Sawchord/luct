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
            if (li.querySelectorAll("ul").length > 0) {
                const headline = li.firstChild;

                const newImage = document.createElement("i");
                newImage.setAttribute("class", "fa fa-angle-right")
                li.insertBefore(newImage, headline);

                const newSpan = document.createElement("span");
                const anchor = li.parentNode;

                anchor.insertBefore(newSpan, li)
                anchor.removeChild(li);
                newSpan.appendChild(li);

                newSpan.addEventListener("click", (event) => {
                    console.log(event);

                    const ul = li.querySelector("ul");
                    if (ul.style.display === "none") {
                        ul.style.display = "block";
                    } else {
                        ul.style.display = "none";
                    }

                    console.log(ul);
                });
            }
        }

    }

    static define() {
        customElements.define("tree-view", TreeView, { extends: "ul" })
    }
}