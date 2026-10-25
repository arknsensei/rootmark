// Bundles the P2-B e2e for Node (e2e/p2_b/.build/run.mjs); not part of the
// plugin bundle.
import esbuild from "esbuild";

await esbuild.build({
  entryPoints: ["e2e/p2_b/run.ts"],
  bundle: true,
  format: "esm",
  platform: "node",
  target: "node22",
  outfile: "e2e/p2_b/.build/run.mjs",
  logLevel: "warning",
});
