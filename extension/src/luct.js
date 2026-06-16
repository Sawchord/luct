import "./style.sass";

import Sidebar from "./views/Sidebar.svelte";
import Options from "./views/Options.svelte";
import Dashboard from "./views/Dashboard.svelte";

export function sidebar() {
    new Sidebar({ target: document.body })
}

export function options() {
    new Options({ target: document.body })
}

export function dashboard() {
    new Dashboard({ target: document.body })
}