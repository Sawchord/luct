import svelte from "rollup-plugin-svelte";
import resolve from "@rollup/plugin-node-resolve";
import terser from "@rollup/plugin-terser";
import livereload from "rollup-plugin-livereload";
import serve from "rollup-plugin-serve";
// import copy from "rollup-plugin-copy";
import css from "rollup-plugin-css-only";
// import fs from "node:fs";
// import path from "node:path";
// import posthtml from "posthtml";
// import { hash } from "posthtml-hash";

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
        PROD && terser(),
    ],
};