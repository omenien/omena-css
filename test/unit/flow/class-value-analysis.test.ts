import ts from "typescript";
import { describe, expect, it } from "vitest";
import { resolveFlowClassValues } from "../../../server/engine-core-ts/src/core/source-frontend/ts-flow-class-value-oracle";
import { buildFlowBlockGraphSnapshot } from "../../../server/engine-core-ts/src/core/source-frontend/ts-source-cfg-oracle";
import { buildFlowSlice } from "../../../server/engine-core-ts/src/core/source-frontend/ts-flow-slice-oracle";

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

describe("buildFlowBlockGraphSnapshot", () => {
  it.each([
    ["&&", 'flag && "is-active"', "logicalAnd"],
    ["||", 'sizeInput || "fallback"', "logicalOr"],
    ["??", 'sizeInput ?? "fallback"', "nullishCoalesce"],
  ] as const)("captures %s short-circuit skip and rhs edges", (_operator, expression, kind) => {
    const source = `
function render(flag: boolean, sizeInput: string | null) {
  const size = ${expression};
  return cx(size);
}
`;

    const graph = flowBlockGraphFor(source);

    expect(graph.blocks).toHaveLength(6);
    expect(blockSummary(graph.blocks)).toEqual([
      ["entry", "entry", ["assignment:0"], undefined],
      ["assignment:0", "assignment", ["logicalOperand:0"], undefined],
      ["logicalOperand:0", "logicalOperand", ["logicalJoin:0", "logicalRhs:0"], kind],
      ["logicalRhs:0", "logicalRhs", ["logicalJoin:0"], kind],
      ["logicalJoin:0", "logicalJoin", ["exit"], kind],
      ["exit", "exit", [], undefined],
    ]);
  });

  it("captures a while-loop header exit edge and body back-edge", () => {
    const source = `
function render(flag: boolean) {
  let size = "base";
  while (flag) {
    size = "loop";
  }
  return cx(size);
}
`;

    const graph = flowBlockGraphFor(source);

    expect(graph.blocks).toHaveLength(7);
    expect(blockSummary(graph.blocks)).toEqual([
      ["entry", "entry", ["assignment:0"], undefined],
      ["assignment:0", "assignment", ["loop:0:header"], undefined],
      ["loop:0:header", "loopHeader", ["loop:0:body", "loop:0:exit"], undefined],
      ["loop:0:body", "loopBody", ["assignment:1"], undefined],
      ["loop:0:exit", "loopExit", ["exit"], undefined],
      ["assignment:1", "assignment", ["loop:0:header"], undefined],
      ["exit", "exit", [], undefined],
    ]);
  });

  it("routes a labeled break block to the labeled loop exit", () => {
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

    const graph = flowBlockGraphFor(source);

    expect(graph.blocks).toHaveLength(8);
    expect(blockSummary(graph.blocks)).toEqual([
      ["entry", "entry", ["assignment:0"], undefined],
      ["assignment:0", "assignment", ["loop:0:header"], undefined],
      ["loop:0:header", "loopHeader", ["loop:0:body", "loop:0:exit"], undefined],
      ["loop:0:body", "loopBody", ["assignment:1"], undefined],
      ["loop:0:exit", "loopExit", ["exit"], undefined],
      ["assignment:1", "assignment", ["break:0"], undefined],
      ["break:0", "break", ["loop:0:exit"], undefined],
      ["exit", "exit", [], undefined],
    ]);
  });
});

function flowBlockGraphFor(source: string) {
  const sourceFile = ts.createSourceFile(
    "/fake/Flow.tsx",
    source,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TSX,
  );
  const slice = buildFlowSlice(sourceFile, rangeOf(source, "cx(size)"), "size");
  if (!slice) throw new Error("expected flow slice");
  return buildFlowBlockGraphSnapshot(slice.nodes);
}

function blockSummary(
  blocks: readonly ReturnType<typeof buildFlowBlockGraphSnapshot>["blocks"][number][],
) {
  return blocks.map(
    (block) => [block.id, block.kind, block.successorBlockIds, block.expressionKind] as const,
  );
}

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
