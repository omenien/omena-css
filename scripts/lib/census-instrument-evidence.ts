import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { compilerApi as ts } from "../../server/engine-core-ts/src/ts-facade";
import type tsTypes from "../../server/engine-core-ts/src/ts-facade";

export const RETIRED_INSTRUMENT = {
  pin: "cfaf03e5a09fd6e0a5f5293c30b44903411f1af4",
  path: "scripts/check-rust-precision-floor.ts",
} as const;

// The runner contract owns this population independently of the editable recipes.
export const CENSUS_ROW_IDS = [
  "a1",
  "a2",
  "b1",
  "b2",
  "c",
  "d",
  "e",
  "f",
  "g1",
  "g2",
  "s1",
  "s2",
  "s3",
  "j",
  "k",
  "l",
  "m",
  "n1",
  "n2",
  "o",
  "p",
  "q",
  "t1",
  "t2",
  "t3",
] as const;

export function assertCensusPopulation(ids: readonly string[]): void {
  const expected = new Set<string>(CENSUS_ROW_IDS);
  const actual = new Set(ids);
  assert.equal(actual.size, ids.length, "duplicate census row id");
  for (const id of expected) assert.ok(actual.has(id), "census row missing " + id);
  for (const id of actual) assert.ok(expected.has(id), "unexpected census row " + id);
  assert.equal(ids.length, CENSUS_ROW_IDS.length, "census row count diverged");
}

function parsedSource(source: string): tsTypes.SourceFile {
  return ts.createSourceFile(
    "instrument.ts",
    source,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
}

export function checkerInventory(
  checkerPath: string,
  source: string,
): Array<{ readonly identity: string; readonly kind: "assert" | "blockBody" }> {
  const file = parsedSource(source);
  const rows: Array<{ identity: string; kind: "assert" | "blockBody" }> = [];
  const visit = (node: tsTypes.Node): void => {
    let kind: "assert" | "blockBody" | undefined;
    if (ts.isCallExpression(node)) {
      const callee = node.expression;
      if (
        (ts.isIdentifier(callee) && callee.text === "assert") ||
        (ts.isPropertyAccessExpression(callee) &&
          ts.isIdentifier(callee.expression) &&
          callee.expression.text === "assert")
      )
        kind = "assert";
      else if (ts.isIdentifier(callee) && callee.text === "blockBody") kind = "blockBody";
    } else if (ts.isFunctionDeclaration(node) && node.name?.text === "blockBody") {
      kind = "blockBody";
    }
    if (kind) {
      const start = node.getStart(file);
      const line = file.getLineAndCharacterOfPosition(start).line + 1;
      const digest = createHash("sha256").update(source.slice(start, node.end)).digest("hex");
      rows.push({ identity: checkerPath + ":" + line + "#" + digest, kind });
    }
    ts.forEachChild(node, visit);
  };
  visit(file);
  return rows.toSorted((a, b) => {
    const left = a.kind + "\0" + a.identity;
    const right = b.kind + "\0" + b.identity;
    return left < right ? -1 : left > right ? 1 : 0;
  });
}

export function assertNoLiteralRowSelection(source: string): void {
  const file = parsedSource(source);
  const aliases = new Set<string>();
  const literalAliases = new Set<string>();
  const isFunctionScope = (node: tsTypes.Node): boolean =>
    ts.isFunctionDeclaration(node) ||
    ts.isFunctionExpression(node) ||
    ts.isArrowFunction(node) ||
    ts.isMethodDeclaration(node) ||
    ts.isConstructorDeclaration(node) ||
    ts.isGetAccessorDeclaration(node) ||
    ts.isSetAccessorDeclaration(node);
  const isLexicalScope = (node: tsTypes.Node): boolean =>
    isFunctionScope(node) ||
    ts.isSourceFile(node) ||
    node.kind === ts.SyntaxKind.Block ||
    node.kind === ts.SyntaxKind.CaseBlock ||
    node.kind === ts.SyntaxKind.CatchClause ||
    node.kind === ts.SyntaxKind.ForStatement ||
    node.kind === ts.SyntaxKind.ForInStatement ||
    node.kind === ts.SyntaxKind.ForOfStatement;
  const scopeKey = (node: tsTypes.Node, name: string): string => {
    const hoisted =
      ts.isVariableDeclaration(node) &&
      node.parent.getFirstToken(file)?.kind === ts.SyntaxKind.VarKeyword;
    let scope: tsTypes.Node = node;
    while (
      scope.parent &&
      (hoisted ? !isFunctionScope(scope) && !ts.isSourceFile(scope) : !isLexicalScope(scope))
    )
      scope = scope.parent;
    return scope.kind + ":" + scope.pos + ":" + scope.end + ":" + name;
  };
  const bindingKeys = new Set<string>();
  const collectBindings = (node: tsTypes.Node): void => {
    if (ts.isVariableDeclaration(node) || node.kind === ts.SyntaxKind.Parameter) {
      const name = (node as tsTypes.Node & { readonly name?: tsTypes.Node }).name;
      if (name && ts.isIdentifier(name)) bindingKeys.add(scopeKey(node, name.text));
    }
    ts.forEachChild(node, collectBindings);
  };
  collectBindings(file);
  const hasAlias = (values: ReadonlySet<string>, node: tsTypes.Identifier): boolean => {
    let cursor: tsTypes.Node | undefined = node;
    while (cursor) {
      const key = scopeKey(cursor, node.text);
      if (bindingKeys.has(key)) return values.has(key);
      let scope: tsTypes.Node = cursor;
      while (scope.parent && !isLexicalScope(scope)) scope = scope.parent;
      cursor = scope.parent;
    }
    return false;
  };
  const unwrap = (node: tsTypes.Node): tsTypes.Node =>
    ts.isParenthesizedExpression(node) ||
    ts.isAsExpression(node) ||
    ts.isNonNullExpression(node) ||
    ts.isTypeAssertionExpression(node)
      ? unwrap(node.expression)
      : node;
  const contains = (
    node: tsTypes.Node,
    predicate: (candidate: tsTypes.Node) => boolean,
  ): boolean => {
    if (predicate(unwrap(node))) return true;
    let found = false;
    ts.forEachChild(node, (child) => {
      if (contains(child, predicate)) found = true;
    });
    return found;
  };
  const selector = (node: tsTypes.Node): boolean => {
    if (ts.isIdentifier(node)) return node.text === "rowId" || hasAlias(aliases, node);
    if (ts.isPropertyAccessExpression(node)) {
      return (
        node.name.text === "refusal" ||
        node.name.text === "refusalPrefix" ||
        (node.name.text === "id" &&
          ts.isIdentifier(unwrap(node.expression)) &&
          (unwrap(node.expression) as tsTypes.Identifier).text === "row")
      );
    }
    if (ts.isElementAccessExpression(node)) {
      const key = node.argumentExpression && unwrap(node.argumentExpression);
      return (
        !!key &&
        (ts.isStringLiteral(key) || ts.isNoSubstitutionTemplateLiteral(key)) &&
        ["id", "refusal", "refusalPrefix"].includes(key.text)
      );
    }
    return false;
  };
  const literalValue = (input: tsTypes.Node): boolean => {
    const node = unwrap(input);
    if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) return true;
    if (ts.isIdentifier(node)) return hasAlias(literalAliases, node);
    if (ts.isArrayLiteralExpression(node)) return node.elements.every(literalValue);
    if (ts.isTemplateExpression(node))
      return node.templateSpans.every((span) => literalValue(span.expression));
    return (
      node.kind === ts.SyntaxKind.NumericLiteral ||
      node.kind === ts.SyntaxKind.TrueKeyword ||
      node.kind === ts.SyntaxKind.FalseKeyword
    );
  };
  // Follow local aliases before inspecting operators, including parenthesized selectors.
  let changed = true;
  while (changed) {
    changed = false;
    const collect = (node: tsTypes.Node): void => {
      if (
        ts.isVariableDeclaration(node) &&
        ts.isIdentifier(node.name) &&
        node.initializer &&
        contains(node.initializer, selector) &&
        !aliases.has(scopeKey(node, node.name.text))
      ) {
        aliases.add(scopeKey(node, node.name.text));
        changed = true;
      }
      if (
        ts.isVariableDeclaration(node) &&
        ts.isIdentifier(node.name) &&
        node.initializer &&
        node.parent.getFirstToken(file)?.kind === ts.SyntaxKind.ConstKeyword &&
        literalValue(node.initializer) &&
        !literalAliases.has(scopeKey(node, node.name.text))
      ) {
        literalAliases.add(scopeKey(node, node.name.text));
        changed = true;
      }
      ts.forEachChild(node, collect);
    };
    collect(file);
  }
  const literal = (node: tsTypes.Node): boolean =>
    ts.isStringLiteral(node) ||
    ts.isNoSubstitutionTemplateLiteral(node) ||
    ts.isTemplateExpression(node) ||
    (ts.isIdentifier(node) && hasAlias(literalAliases, node));
  const equality = new Set([
    ts.SyntaxKind.EqualsEqualsToken,
    ts.SyntaxKind.EqualsEqualsEqualsToken,
    ts.SyntaxKind.ExclamationEqualsToken,
    ts.SyntaxKind.ExclamationEqualsEqualsToken,
  ]);
  const visit = (node: tsTypes.Node): void => {
    let forbidden = false;
    if (ts.isBinaryExpression(node) && equality.has(node.operatorToken.kind)) {
      forbidden =
        (contains(node.left, selector) && contains(node.right, literal)) ||
        (contains(node.right, selector) && contains(node.left, literal));
    } else if (
      ts.isCallExpression(node) &&
      ts.isPropertyAccessExpression(node.expression) &&
      node.expression.name.text === "includes"
    ) {
      forbidden = contains(node, selector) && contains(node, literal);
    } else if (ts.isSwitchStatement(node)) {
      forbidden =
        contains(node.expression, selector) &&
        node.caseBlock.clauses.some(
          (clause) => ts.isCaseClause(clause) && contains(clause.expression, literal),
        );
    }
    const line = file.getLineAndCharacterOfPosition(node.getStart(file)).line + 1;
    assert.ok(!forbidden, "per-row literal selection is forbidden at line " + line);
    ts.forEachChild(node, visit);
  };
  visit(file);
}
