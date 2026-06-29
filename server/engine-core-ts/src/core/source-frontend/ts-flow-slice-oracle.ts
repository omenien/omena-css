import ts, { nodeStart, positionOfLineChar } from "../../ts-facade";
import type { Range } from "@omena/shared";
import { buildFlowNodes, type FlowNode } from "./ts-source-cfg-oracle";

export interface FlowSlice {
  readonly variableName: string;
  readonly referencePos: number;
  readonly container: ts.SourceFile | ts.Block;
  readonly nodes: readonly FlowNode[];
}

export function buildFlowSlice(
  sourceFile: ts.SourceFile,
  range: Range,
  variableName: string,
): FlowSlice | null {
  if (variableName.includes(".")) return null;
  if (range.start.line >= sourceFile.getLineStarts().length) return null;

  const referencePos = positionOfLineChar(sourceFile, range.start);
  const referenceNode = findInnermostContainingNode(sourceFile, referencePos);
  if (!referenceNode) return null;

  const container = findStatementContainer(referenceNode);
  if (!container) return null;
  if (hasAmbiguousDeclarations(container, variableName, referencePos)) return null;

  return {
    variableName,
    referencePos,
    container,
    nodes: buildFlowNodes(container.statements, referencePos),
  };
}

function findInnermostContainingNode(root: ts.Node, pos: number): ts.Node | null {
  if (!(nodeStart(root) <= pos && pos < root.end)) return null;

  let current: ts.Node = root;
  ts.forEachChild(root, (child) => {
    const nested = findInnermostContainingNode(child, pos);
    if (nested) current = nested;
  });
  return current;
}

function findStatementContainer(node: ts.Node): ts.SourceFile | ts.Block | null {
  let current: ts.Node | undefined = node;
  while (current) {
    if (ts.isSourceFile(current)) return current;
    if (ts.isBlock(current) && isFunctionLike(current.parent)) return current;
    current = current.parent;
  }
  return null;
}

function isFunctionLike(node: ts.Node | undefined): node is ts.FunctionLikeDeclaration {
  return (
    !!node &&
    (ts.isFunctionDeclaration(node) ||
      ts.isFunctionExpression(node) ||
      ts.isArrowFunction(node) ||
      ts.isMethodDeclaration(node) ||
      ts.isGetAccessorDeclaration(node) ||
      ts.isSetAccessorDeclaration(node) ||
      ts.isConstructorDeclaration(node))
  );
}

function hasAmbiguousDeclarations(
  container: ts.SourceFile | ts.Block,
  variableName: string,
  referencePos: number,
): boolean {
  let declarations = 0;

  const visit = (node: ts.Node): void => {
    if (nodeStart(node) >= referencePos) return;
    if (isNestedFunctionBody(node, container)) return;

    if (ts.isVariableDeclaration(node) || ts.isParameter(node) || ts.isBindingElement(node)) {
      const name = node.name;
      if (ts.isIdentifier(name) && name.text === variableName) {
        declarations += 1;
      }
    }

    ts.forEachChild(node, visit);
  };

  for (const statement of container.statements) {
    visit(statement);
    if (declarations > 1) return true;
  }

  return false;
}

function isNestedFunctionBody(node: ts.Node, container: ts.SourceFile | ts.Block): boolean {
  if (!isFunctionLike(node)) return false;
  return node.body !== container;
}
