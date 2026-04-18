import svelte from "rollup-plugin-svelte";
import resolve from "@rollup/plugin-node-resolve";
import livereload from "rollup-plugin-livereload";
import serve from "rollup-plugin-serve";
import css from "rollup-plugin-css-only";
import postcss from 'rollup-plugin-postcss'
import { sveltePreprocess } from 'svelte-preprocess'
import copy from "rollup-plugin-copy";

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
            preprocess: sveltePreprocess(),
            compilerOptions: { dev: !PROD },
        }),
        postcss(),
        css({ output: "bundle.css" }),
        copy({
            targets: [{
                src: "node_modules/font-awesome/fonts",
                dest: "luct/assets"
            }]

        }),
        resolve(),
        !PROD &&
        serve({
            contentBase: ["luct"],
            port: 3000,
        }),
        !PROD && livereload({ watch: "luct" }),
    ],
};