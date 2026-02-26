
import DateTime from "./luct/components/datetimes.js";
import TreeView from "./luct/components/treeview.js";
import Report from "./luct/components/report.js";

DateTime.define();
TreeView.define();
Report.define();


let data = await fetch("./test-report.json");
data = await data.json();

const anchor = document.querySelector("#report-here");
const report = new Report(data);
anchor.appendChild(report);
