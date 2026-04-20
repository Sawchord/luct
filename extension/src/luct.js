import "./style.sass";
import Sidebar from "./Sidebar.svelte";

export function sidebar() {
    new Sidebar({ target: document.body })
}
