// Bundles the real-Obsidian e2e for Node (e2e/real/.build/run.mjs); not part
// of the plugin bundle.
import esbuild from "esbuild";

await esbuild.build({
  entryPoints: ["e2e/real/run.ts"],
  bundle: true,
  format: "esm",
  platform: "node",
  target: "node22",
  outfile: "e2e/real/.build/run.mjs",
  logLevel: "warning",
});
