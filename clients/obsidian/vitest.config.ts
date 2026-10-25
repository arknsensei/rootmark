// vitest: every test file runs under the contract checker (P2-B/B4 (c)).
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    setupFiles: ["test/contract-setup.ts"],
    env: { ROOTMARK_CONTRACT_CLIENT: "obsidian" },
  },
});
