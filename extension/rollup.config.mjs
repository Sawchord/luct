import svelte from 'rollup-plugin-svelte';
import resolve from '@rollup/plugin-node-resolve';

export default {
    input: 'src/luct.js',
    output: {
        sourcemap: false,
        format: 'iife',
        name: 'sidebar',
        file: 'luct/assets/bundle.js'
    },
    plugins: [
        svelte(),
        resolve({
            browser: true,
            dedupe: ['svelte']
        }),

    ]
}