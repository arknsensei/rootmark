// Bundles the extension (dist/extension.cjs, CommonJS for the extension
// host); `vscode` is provided by the host. No runtime dependency.
import esbuild from "esbuild";

await esbuild.build({
  entryPoints: ["src/extension.ts"],
  bundle: true,
  format: "cjs",
  platform: "node",
  target: "node20",
  external: ["vscode"],
  outfile: "dist/extension.cjs",
  logLevel: "warning",
});
