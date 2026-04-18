import svelte from "rollup-plugin-svelte";
import resolve from "@rollup/plugin-node-resolve";
import livereload from "rollup-plugin-livereload";
import serve from "rollup-plugin-serve";
import css from "rollup-plugin-css-only";

const PROD = !process.env.ROLLUP_WATCH;

export default {
    input: "src/luct.js",
    output: {
        sourcemap: !PROD,
        format: "iife",
        name: "sidebar",
        file: "luct/assets/bundle.js",
    },
    plugins: [
        svelte({
            compilerOptions: { dev: !PROD },
        }),
        css({ output: "bundle.css" }),
        resolve(),
        !PROD &&
        serve({
            contentBase: ["luct"],
            port: 3000,
        }),
        !PROD && livereload({ watch: "luct" }),
    ],
};