import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  loadExternalSifsForWorkspace,
  type ExternalSifLoaderDeps,
} from "../../../server/engine-host-node/src/external-sif-loader";

const WORKSPACE_ROOT = "/workspace";
const LOCK_PATH = path.join(WORKSPACE_ROOT, "omena.lock");

function makeDeps(files: Record<string, string>): Required<ExternalSifLoaderDeps> {
  return {
    fileExists: (filePath) => Object.prototype.hasOwnProperty.call(files, filePath),
    readFile: (filePath) => {
      const content = files[filePath];
      if (content === undefined) throw new Error(`no such file: ${filePath}`);
      return content;
    },
  };
}

describe("loadExternalSifsForWorkspace", () => {
  it("loads lock entries and forwards { canonicalUrl, sif } verbatim", () => {
    const sifPath = path.join(WORKSPACE_ROOT, ".omena/design-system.sif.json");
    const sif = { sifVersion: "1", canonicalUrl: "pkg:design-system/_tokens.scss" };
    const deps = makeDeps({
      [LOCK_PATH]: JSON.stringify({
        lockfileVersion: "1",
        entries: [
          {
            canonicalUrl: "pkg:design-system/_tokens.scss",
            sifPath: ".omena/design-system.sif.json",
          },
        ],
      }),
      [sifPath]: JSON.stringify(sif),
    });

    const result = loadExternalSifsForWorkspace(WORKSPACE_ROOT, deps);
    expect(result).toEqual([{ canonicalUrl: "pkg:design-system/_tokens.scss", sif }]);
  });

  it("resolves absolute sifPath entries directly", () => {
    const sifPath = "/abs/tokens.sif.json";
    const sif = { canonicalUrl: "pkg:x" };
    const deps = makeDeps({
      [LOCK_PATH]: JSON.stringify({ entries: [{ canonicalUrl: "pkg:x", sifPath }] }),
      [sifPath]: JSON.stringify(sif),
    });
    expect(loadExternalSifsForWorkspace(WORKSPACE_ROOT, deps)).toEqual([
      { canonicalUrl: "pkg:x", sif },
    ]);
  });

  it("returns [] when the workspace root is undefined (Ignored behaviour)", () => {
    expect(loadExternalSifsForWorkspace(undefined, makeDeps({}))).toEqual([]);
  });

  it("returns [] when no omena.lock exists (Ignored behaviour)", () => {
    expect(loadExternalSifsForWorkspace(WORKSPACE_ROOT, makeDeps({}))).toEqual([]);
  });

  it("skips entries whose SIF artifact is missing", () => {
    const deps = makeDeps({
      [LOCK_PATH]: JSON.stringify({
        entries: [{ canonicalUrl: "pkg:x", sifPath: "missing.sif.json" }],
      }),
    });
    expect(loadExternalSifsForWorkspace(WORKSPACE_ROOT, deps)).toEqual([]);
  });

  it("returns [] on malformed lock JSON rather than throwing", () => {
    const deps = makeDeps({ [LOCK_PATH]: "{ not json" });
    expect(loadExternalSifsForWorkspace(WORKSPACE_ROOT, deps)).toEqual([]);
  });

  it("skips entries whose SIF artifact is malformed JSON", () => {
    const sifPath = path.join(WORKSPACE_ROOT, "bad.sif.json");
    const deps = makeDeps({
      [LOCK_PATH]: JSON.stringify({
        entries: [{ canonicalUrl: "pkg:x", sifPath: "bad.sif.json" }],
      }),
      [sifPath]: "{ not json",
    });
    expect(loadExternalSifsForWorkspace(WORKSPACE_ROOT, deps)).toEqual([]);
  });
});
