import svelte from "rollup-plugin-svelte";
import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import livereload from "rollup-plugin-livereload";
import wasm from "@wasm-tool/rollup-plugin-rust";
import { terser } from "rollup-plugin-terser";

const production = !process.env.ROLLUP_WATCH;

export default {
    input: {
        main: "src/main.js",
    },
    output: {
        format: "es",
        name: "app",
        dir: "public/build/",
    },
    plugins: [
        svelte({
            dev: !production,
            css: (css) => {
                css.write("public/build/main.css");
            },
        }),

        resolve({
            browser: true,
            dedupe: ["svelte"],
        }),
        commonjs(),
        wasm({
            verbose: true,
            serverPath: "build/",
        }),

        !production && serve(),
        !production && livereload("public"),

        production && terser(),
    ],
    watch: {
        clearScreen: false,
    },
};

function serve() {
    let started = false;

    return {
        writeBundle() {
            if (!started) {
                started = true;

                require("child_process").spawn(
                    "npm",
                    ["run", "start", "--", "--dev"],
                    {
                        stdio: ["ignore", "inherit", "inherit"],
                        shell: true,
                    }
                );
            }
        },
    };
}
