// Bundles src/main.ts into main.js (CommonJS, as Obsidian loads plugins).
// `obsidian` and CodeMirror are provided by the app at run time: they are
// external, never bundled; nothing else may be imported at run time
// (GOAL P1-E E-10: no runtime dependency besides the Obsidian API).
import esbuild from "esbuild";

await esbuild.build({
  entryPoints: ["src/main.ts"],
  bundle: true,
  format: "cjs",
  platform: "browser",
  target: "es2022",
  outfile: "main.js",
  external: ["obsidian", "electron", "@codemirror/state", "@codemirror/view"],
  legalComments: "none",
  sourcemap: false,
  minify: false,
  logLevel: "warning",
  metafile: false,
});
