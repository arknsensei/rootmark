import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const root = new URL("..", import.meta.url);
const json = (f: string): Record<string, unknown> =>
  JSON.parse(readFileSync(new URL(f, root), "utf8")) as Record<string, unknown>;

describe("package (GOAL P1-E E-10)", () => {
  it("has no runtime dependency: everything but the Obsidian API is a dev dependency", () => {
    const p = json("package.json");
    expect(p["dependencies"]).toBeUndefined();
    expect(p["peerDependencies"]).toBeUndefined();
    expect(p["optionalDependencies"]).toBeUndefined();
  });

  it("pins every dev dependency to an exact version", () => {
    const dev = json("package.json")["devDependencies"] as Record<string, string>;
    for (const [name, v] of Object.entries(dev)) {
      expect(v, name).toMatch(/^\d+\.\d+\.\d+$/);
    }
  });

  it("manifest and package carry one version", () => {
    expect(json("manifest.json")["version"]).toBe(json("package.json")["version"]);
  });
});
