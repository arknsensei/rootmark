// Bundles the VS Code e2e for Node (e2e/.build/run.mjs); not part of the
// extension bundle.
import esbuild from "esbuild";

await esbuild.build({
  entryPoints: ["e2e/run.ts"],
  bundle: true,
  format: "esm",
  platform: "node",
  target: "node22",
  outfile: "e2e/.build/run.mjs",
  logLevel: "warning",
});
