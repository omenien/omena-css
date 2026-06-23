import ts from "typescript";

export type FlowNode =
  | AssignmentFlowNode
  | BranchFlowNode
  | LoopFlowNode
  | BreakFlowNode
  | TerminateFlowNode;

export interface AssignmentFlowNode {
  readonly kind: "assignment";
  readonly statement: ts.Statement;
  readonly variableName: string;
  readonly expression: ts.Expression | null;
}

export interface BranchFlowNode {
  readonly kind: "branch";
  readonly statement: ts.IfStatement;
  readonly referenceLocation: "then" | "else" | "after";
  readonly thenNodes: readonly FlowNode[];
  readonly elseNodes: readonly FlowNode[];
}

export interface LoopFlowNode {
  readonly kind: "loop";
  readonly statement: ts.WhileStatement | ts.ForStatement | ts.DoStatement;
  readonly referenceLocation: "body" | "after";
  readonly bodyNodes: readonly FlowNode[];
}

export interface BreakFlowNode {
  readonly kind: "break";
  readonly statement: ts.BreakStatement;
}

export interface TerminateFlowNode {
  readonly kind: "terminate";
  readonly statement: ts.ReturnStatement | ts.ThrowStatement;
}

export function buildFlowNodes(
  statements: readonly ts.Statement[],
  referencePos: number,
): readonly FlowNode[] {
  const nodes: FlowNode[] = [];

  for (const statement of statements) {
    if (statement.getStart() >= referencePos) break;

    if (ts.isFunctionDeclaration(statement)) continue;

    if (ts.isIfStatement(statement)) {
      const referenceLocation = locateReferenceInIf(statement, referencePos);
      nodes.push({
        kind: "branch",
        statement,
        referenceLocation,
        thenNodes: buildFlowNodes(
          statementListOf(statement.thenStatement),
          branchReferencePos(referenceLocation, "then", referencePos),
        ),
        elseNodes: buildFlowNodes(
          statementListOf(statement.elseStatement),
          branchReferencePos(referenceLocation, "else", referencePos),
        ),
      });
      if (referenceLocation !== "after") break;
      continue;
    }

    if (isLoopStatement(statement)) {
      const referenceLocation = containsPosition(statement.statement, referencePos)
        ? "body"
        : "after";
      nodes.push({
        kind: "loop",
        statement,
        referenceLocation,
        bodyNodes: buildFlowNodes(
          statementListOf(statement.statement),
          referenceLocation === "body" ? referencePos : Number.POSITIVE_INFINITY,
        ),
      });
      if (referenceLocation === "body") break;
      continue;
    }

    if (ts.isLabeledStatement(statement)) {
      nodes.push(...buildFlowNodes([statement.statement], referencePos));
      if (containsPosition(statement.statement, referencePos)) break;
      continue;
    }

    if (ts.isBreakStatement(statement)) {
      nodes.push({ kind: "break", statement });
      break;
    }

    if (ts.isReturnStatement(statement) || ts.isThrowStatement(statement)) {
      nodes.push({ kind: "terminate", statement });
      break;
    }

    nodes.push(...assignmentNodesForStatement(statement));
  }

  return nodes;
}

function isLoopStatement(
  statement: ts.Statement,
): statement is ts.WhileStatement | ts.ForStatement | ts.DoStatement {
  return (
    ts.isWhileStatement(statement) || ts.isForStatement(statement) || ts.isDoStatement(statement)
  );
}

function statementListOf(statement: ts.Statement | undefined): readonly ts.Statement[] {
  if (!statement) return [];
  if (ts.isBlock(statement)) return statement.statements;
  return [statement];
}

function locateReferenceInIf(
  statement: ts.IfStatement,
  referencePos: number,
): BranchFlowNode["referenceLocation"] {
  if (containsPosition(statement.thenStatement, referencePos)) return "then";
  if (statement.elseStatement && containsPosition(statement.elseStatement, referencePos))
    return "else";
  return "after";
}

function branchReferencePos(
  referenceLocation: BranchFlowNode["referenceLocation"],
  branch: "then" | "else",
  referencePos: number,
): number {
  return referenceLocation === branch ? referencePos : Number.POSITIVE_INFINITY;
}

function assignmentNodesForStatement(statement: ts.Statement): readonly AssignmentFlowNode[] {
  if (ts.isVariableStatement(statement)) {
    return statement.declarationList.declarations.flatMap((declaration) => {
      if (!ts.isIdentifier(declaration.name)) return [];
      return [
        {
          kind: "assignment",
          statement,
          variableName: declaration.name.text,
          expression: declaration.initializer ?? null,
        } satisfies AssignmentFlowNode,
      ];
    });
  }

  if (ts.isExpressionStatement(statement) && ts.isBinaryExpression(statement.expression)) {
    const expr = statement.expression;
    if (expr.operatorToken.kind === ts.SyntaxKind.EqualsToken && ts.isIdentifier(expr.left)) {
      return [
        {
          kind: "assignment",
          statement,
          variableName: expr.left.text,
          expression: expr.right,
        } satisfies AssignmentFlowNode,
      ];
    }
  }

  if (ts.isBlock(statement)) {
    return buildFlowNodes(statement.statements, Number.POSITIVE_INFINITY).flatMap((node) =>
      node.kind === "assignment" ? [node] : [],
    );
  }

  return [];
}

function containsPosition(node: ts.Node, pos: number): boolean {
  return node.getStart() <= pos && pos < node.end;
}
