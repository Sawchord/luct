import "./style.sass";

import Sidebar from "./Sidebar.svelte";
import Options from "./Options.svelte"

export function sidebar() {
    new Sidebar({ target: document.body })
}

export function options() {
    new Options({ target: document.body })
}
