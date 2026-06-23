import ts from "typescript";
import { describe, expect, it } from "vitest";
import { resolveFlowClassValues } from "../../../server/engine-core-ts/src/core/flow/class-value-analysis";

describe("resolveFlowClassValues", () => {
  it("tracks straight-line reassignment before the class use", () => {
    const source = `
function render() {
  let size = "sm";
  size = "lg";
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "exact",
        value: "lg",
      },
      valueCertainty: "exact",
      reason: "flowLiteral",
    });
  });

  it("merges branch-local assignments into an inferred union", () => {
    const source = `
function render(flag: boolean) {
  let size = "sm";
  if (flag) {
    size = "lg";
  }
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "finiteSet",
        values: ["lg", "sm"],
      },
      valueCertainty: "inferred",
      reason: "flowBranch",
    });
  });

  it("prunes branches that return before the class use", () => {
    const source = `
function render(flag: boolean) {
  let size = "sm";
  if (flag) {
    size = "lg";
    return null;
  }
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "exact",
        value: "sm",
      },
      valueCertainty: "exact",
      reason: "flowLiteral",
    });
  });

  it("derives a prefix domain from concatenation with an unknown suffix", () => {
    const source = `
function render(variant: string) {
  const size = "btn-" + variant;
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "prefix",
        prefix: "btn-",
        provenance: "concatUnknownRight",
      },
      valueCertainty: "inferred",
      reason: "flowLiteral",
    });
  });

  it("derives a suffix domain from concatenation with an unknown prefix", () => {
    const source = `
function render(variant: string) {
  const size = variant + "-chip";
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "suffix",
        suffix: "-chip",
        provenance: "concatUnknownLeft",
      },
      valueCertainty: "inferred",
      reason: "flowLiteral",
    });
  });

  it("derives a prefix-suffix domain from known prefix plus unknown middle plus known suffix", () => {
    const source = `
function render(variant: string) {
  const size = "btn-" + variant + "-chip";
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "prefixSuffix",
        prefix: "btn-",
        suffix: "-chip",
        minLength: 9,
        provenance: "concatKnownEdges",
      },
      valueCertainty: "inferred",
      reason: "flowLiteral",
    });
  });

  it("widens conflicting concatenation prefixes to top", () => {
    const source = `
function render(flag: boolean, variant: string) {
  const prefix = flag ? "btn-" : "card-";
  const size = prefix + variant;
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "top",
      },
      valueCertainty: "possible",
      reason: "flowBranch",
    });
  });

  it("preserves string candidates from logical-and short-circuit expressions", () => {
    const source = `
function render(flag: boolean) {
  const size = flag && "is-active";
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "exact",
        value: "is-active",
      },
      valueCertainty: "exact",
      reason: "flowBranch",
    });
  });

  it("preserves fallback candidates from nullish short-circuit expressions", () => {
    const source = `
function render(sizeInput: string | null) {
  const size = sizeInput ?? "fallback";
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "exact",
        value: "fallback",
      },
      valueCertainty: "exact",
      reason: "flowBranch",
    });
  });

  it("merges while-loop assignments with the zero-iteration path", () => {
    const source = `
function render(flag: boolean) {
  let size = "base";
  while (flag) {
    size = "loop";
    break;
  }
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "finiteSet",
        values: ["base", "loop"],
      },
      valueCertainty: "inferred",
      reason: "flowBranch",
    });
  });

  it("merges for-loop assignments with the zero-iteration path", () => {
    const source = `
function render(count: number) {
  let size = "base";
  for (let index = 0; index < count; index += 1) {
    size = "loop";
  }
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "finiteSet",
        values: ["base", "loop"],
      },
      valueCertainty: "inferred",
      reason: "flowBranch",
    });
  });

  it("routes labeled break to the loop exit without consuming dead assignments", () => {
    const source = `
function render(flag: boolean) {
  let size = "base";
  outer: while (flag) {
    size = "loop";
    break outer;
    size = "never";
  }
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "finiteSet",
        values: ["base", "loop"],
      },
      valueCertainty: "inferred",
      reason: "flowBranch",
    });
  });

  it("derives a finite set from a same-file helper call that returns string literals", () => {
    const source = `
type Status = "idle" | "busy" | "error";

function resolveStatusClass(status: Status): string {
  switch (status) {
    case "idle":
      return "state-idle";
    case "busy":
      return "state-busy";
    case "error":
      return "state-error";
    default:
      return "state-idle";
  }
}

function render(status: Status) {
  const derivedStatusClass = resolveStatusClass(status);
  return cx(derivedStatusClass);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(
      resolveFlowClassValues(
        sourceFile,
        rangeOf(source, "cx(derivedStatusClass)"),
        "derivedStatusClass",
      ),
    ).toEqual({
      abstractValue: {
        kind: "finiteSet",
        values: ["state-busy", "state-error", "state-idle"],
      },
      valueCertainty: "inferred",
      reason: "flowBranch",
    });
  });

  it("widens a large same-file helper literal set to character inclusion constraints", () => {
    const source = `
function resolveSize(flag: number): string {
  switch (flag) {
    case 0: return "a-0";
    case 1: return "a-1";
    case 2: return "a-2";
    case 3: return "b-0";
    case 4: return "b-1";
    case 5: return "b-2";
    case 6: return "c-0";
    case 7: return "c-1";
    default: return "c-2";
  }
}

function render(flag: number) {
  const size = resolveSize(flag);
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "charInclusion",
        mustChars: "-",
        mayChars: "-012abc",
        provenance: "finiteSetWideningChars",
      },
      valueCertainty: "inferred",
      reason: "flowBranch",
    });
  });

  it("widens a large same-file helper literal set with shared prefix to a composite domain", () => {
    const source = `
function resolveSize(flag: number): string {
  switch (flag) {
    case 0: return "btn-0";
    case 1: return "btn-1";
    case 2: return "btn-2";
    case 3: return "btn-3";
    case 4: return "btn-4";
    case 5: return "btn-5";
    case 6: return "btn-6";
    case 7: return "btn-7";
    default: return "btn-8";
  }
}

function render(flag: number) {
  const size = resolveSize(flag);
  return cx(size);
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/Flow.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );

    expect(resolveFlowClassValues(sourceFile, rangeOf(source, "cx(size)"), "size")).toEqual({
      abstractValue: {
        kind: "composite",
        prefix: "btn-",
        minLength: 5,
        mustChars: "-bnt",
        mayChars: "-012345678bnt",
        provenance: "finiteSetWideningComposite",
      },
      valueCertainty: "inferred",
      reason: "flowBranch",
    });
  });
});

function rangeOf(source: string, token: string) {
  const tokenIndex = source.lastIndexOf(token);
  const startIndex = tokenIndex + token.indexOf("size");
  const prefix = source.slice(0, startIndex);
  const line = prefix.split("\n").length - 1;
  const lastLineStart = prefix.lastIndexOf("\n");
  const character = startIndex - (lastLineStart + 1);
  return {
    start: { line, character },
    end: { line, character: character + 4 },
  };
}
