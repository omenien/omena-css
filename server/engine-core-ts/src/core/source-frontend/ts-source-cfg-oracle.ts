import ts, { nodeStart } from "../../ts-facade";

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

export interface FlowBlockGraphSnapshot {
  readonly entryBlockId: string;
  readonly blocks: readonly FlowBlockSnapshot[];
}

export interface FlowBlockSnapshot {
  readonly id: string;
  readonly kind:
    | "entry"
    | "assignment"
    | "branch"
    | "join"
    | "loopHeader"
    | "loopBody"
    | "loopExit"
    | "break"
    | "terminate"
    | "logicalOperand"
    | "logicalRhs"
    | "logicalJoin"
    | "exit";
  readonly transferKind:
    | "entry"
    | "assignFacts"
    | "branch"
    | "concatFacts"
    | "join"
    | "loop"
    | "break"
    | "terminate"
    | "exit";
  readonly successorBlockIds: readonly string[];
  readonly boundaryEffect: "concatInsideToken" | "concatAtTokenBoundary" | "unknownBoundary";
  readonly variableName?: string;
  readonly expressionKind?: "logicalAnd" | "logicalOr" | "nullishCoalesce";
}

export function buildFlowNodes(
  statements: readonly ts.Statement[],
  referencePos: number,
): readonly FlowNode[] {
  const nodes: FlowNode[] = [];

  for (const statement of statements) {
    if (nodeStart(statement) >= referencePos) break;

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

    if (containsPosition(statement, referencePos)) break;

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

export function buildFlowBlockGraphSnapshot(nodes: readonly FlowNode[]): FlowBlockGraphSnapshot {
  const builder = new FlowBlockGraphSnapshotBuilder();
  return builder.build(nodes);
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
  return nodeStart(node) <= pos && pos < node.end;
}

class FlowBlockGraphSnapshotBuilder {
  readonly #blocks: MutableFlowBlockSnapshot[] = [];
  readonly #counters = new Map<string, number>();

  build(nodes: readonly FlowNode[]): FlowBlockGraphSnapshot {
    const entryBlockId = this.#addBlock("entry", "entry");
    const tails = this.#appendNodes(nodes, [entryBlockId], {});
    const exitBlockId = this.#addBlock("exit", "exit");
    this.#connect(tails, exitBlockId);
    return {
      entryBlockId,
      blocks: this.#blocks.map((block) => ({
        ...block,
        successorBlockIds: [...block.successorBlockIds],
      })),
    };
  }

  #appendNodes(
    nodes: readonly FlowNode[],
    incomingBlockIds: readonly string[],
    context: FlowBlockContext,
  ): readonly string[] {
    let tails = [...incomingBlockIds];

    for (const node of nodes) {
      if (tails.length === 0) return [];
      tails = [...this.#appendNode(node, tails, context)];
    }

    return tails;
  }

  #appendNode(
    node: FlowNode,
    incomingBlockIds: readonly string[],
    context: FlowBlockContext,
  ): readonly string[] {
    switch (node.kind) {
      case "assignment":
        return this.#appendAssignment(node, incomingBlockIds);
      case "branch":
        return this.#appendBranch(node, incomingBlockIds, context);
      case "loop":
        return this.#appendLoop(node, incomingBlockIds, context);
      case "break": {
        const breakBlockId = this.#addBlock("break");
        this.#connect(incomingBlockIds, breakBlockId);
        if (context.breakTargetBlockId) {
          this.#connect([breakBlockId], context.breakTargetBlockId);
        }
        return [];
      }
      case "terminate": {
        const terminateBlockId = this.#addBlock("terminate");
        this.#connect(incomingBlockIds, terminateBlockId);
        return [];
      }
      default:
        node satisfies never;
        return incomingBlockIds;
    }
  }

  #appendAssignment(
    node: AssignmentFlowNode,
    incomingBlockIds: readonly string[],
  ): readonly string[] {
    const assignmentBlockId = this.#addBlock("assignment", undefined, {
      transferKind: isConcatExpression(node.expression) ? "concatFacts" : "assignFacts",
      variableName: node.variableName,
      boundaryEffect: classBoundaryEffectForExpression(node.expression),
    });
    this.#connect(incomingBlockIds, assignmentBlockId);

    if (node.expression && isShortCircuitExpression(node.expression)) {
      return this.#appendShortCircuitExpression(node.expression, [assignmentBlockId]);
    }

    return [assignmentBlockId];
  }

  #appendShortCircuitExpression(
    expression: ts.BinaryExpression,
    incomingBlockIds: readonly string[],
  ): readonly string[] {
    const expressionKind = shortCircuitExpressionKind(expression);
    const operandBlockId = this.#addBlock("logicalOperand", undefined, { expressionKind });
    const rhsBlockId = this.#addBlock("logicalRhs", undefined, { expressionKind });
    const joinBlockId = this.#addBlock("logicalJoin", undefined, { expressionKind });
    this.#connect(incomingBlockIds, operandBlockId);
    this.#connect([operandBlockId], joinBlockId);
    this.#connect([operandBlockId], rhsBlockId);
    this.#connect([rhsBlockId], joinBlockId);
    return [joinBlockId];
  }

  #appendBranch(
    node: BranchFlowNode,
    incomingBlockIds: readonly string[],
    context: FlowBlockContext,
  ): readonly string[] {
    const branchBlockId = this.#addBlock("branch");
    const joinBlockId = this.#addBlock("join");
    this.#connect(incomingBlockIds, branchBlockId);

    const thenTails = this.#appendNodes(node.thenNodes, [branchBlockId], context);
    const elseTails =
      node.elseNodes.length > 0
        ? this.#appendNodes(node.elseNodes, [branchBlockId], context)
        : [branchBlockId];
    this.#connect(thenTails, joinBlockId);
    this.#connect(elseTails, joinBlockId);
    return [joinBlockId];
  }

  #appendLoop(
    node: LoopFlowNode,
    incomingBlockIds: readonly string[],
    context: FlowBlockContext,
  ): readonly string[] {
    const loopIndex = this.#nextIndex("loop");
    const headerBlockId = `loop:${loopIndex}:header`;
    const bodyBlockId = `loop:${loopIndex}:body`;
    const exitBlockId = `loop:${loopIndex}:exit`;
    this.#addBlock("loopHeader", headerBlockId);
    this.#addBlock("loopBody", bodyBlockId);
    this.#addBlock("loopExit", exitBlockId);
    this.#connect(incomingBlockIds, headerBlockId);
    this.#connect([headerBlockId], bodyBlockId);
    this.#connect([headerBlockId], exitBlockId);

    const bodyTails = this.#appendNodes(node.bodyNodes, [bodyBlockId], {
      ...context,
      breakTargetBlockId: exitBlockId,
    });
    this.#connect(bodyTails, headerBlockId);
    return [exitBlockId];
  }

  #addBlock(
    kind: FlowBlockSnapshot["kind"],
    explicitId?: string,
    metadata: Omit<Partial<MutableFlowBlockSnapshot>, "id" | "kind" | "successorBlockIds"> = {},
  ): string {
    const id = explicitId ?? `${kind}:${this.#nextIndex(kind)}`;
    this.#blocks.push({
      id,
      kind,
      transferKind: metadata.transferKind ?? transferKindForBlockKind(kind),
      successorBlockIds: [],
      boundaryEffect: metadata.boundaryEffect ?? "unknownBoundary",
      ...metadata,
    });
    return id;
  }

  #connect(fromBlockIds: readonly string[], toBlockId: string): void {
    for (const fromBlockId of fromBlockIds) {
      const block = this.#blocks.find((candidate) => candidate.id === fromBlockId);
      if (!block || block.successorBlockIds.includes(toBlockId)) continue;
      block.successorBlockIds.push(toBlockId);
    }
  }

  #nextIndex(kind: string): number {
    const next = this.#counters.get(kind) ?? 0;
    this.#counters.set(kind, next + 1);
    return next;
  }
}

interface FlowBlockContext {
  readonly breakTargetBlockId?: string;
}

type MutableFlowBlockSnapshot = Omit<FlowBlockSnapshot, "successorBlockIds"> & {
  successorBlockIds: string[];
};

function isShortCircuitExpression(expression: ts.Expression): expression is ts.BinaryExpression {
  return (
    ts.isBinaryExpression(expression) &&
    (expression.operatorToken.kind === ts.SyntaxKind.AmpersandAmpersandToken ||
      expression.operatorToken.kind === ts.SyntaxKind.BarBarToken ||
      expression.operatorToken.kind === ts.SyntaxKind.QuestionQuestionToken)
  );
}

function isConcatExpression(expression: ts.Expression | null): boolean {
  if (!expression) return false;
  const unwrapped = transparentExpression(expression);
  if (
    ts.isBinaryExpression(unwrapped) &&
    unwrapped.operatorToken.kind === ts.SyntaxKind.PlusToken
  ) {
    return true;
  }
  if (ts.isTemplateExpression(unwrapped)) return true;
  return (
    ts.isCallExpression(unwrapped) && classBoundaryEffectForCall(unwrapped) !== "unknownBoundary"
  );
}

function classBoundaryEffectForExpression(
  expression: ts.Expression | null,
): FlowBlockSnapshot["boundaryEffect"] {
  if (!expression) return "unknownBoundary";
  const unwrapped = transparentExpression(expression);
  if (
    ts.isBinaryExpression(unwrapped) &&
    unwrapped.operatorToken.kind === ts.SyntaxKind.PlusToken
  ) {
    return boundaryEffectForBinaryConcat(unwrapped.left, unwrapped.right);
  }
  if (ts.isTemplateExpression(unwrapped)) {
    let sawLiteralDelimiter = unwrapped.head.text.length > 0;
    let leftLiteral = unwrapped.head.text;
    for (const span of unwrapped.templateSpans) {
      if (
        endsWithDomClassAsciiWhitespace(leftLiteral) ||
        startsWithDomClassAsciiWhitespace(span.literal.text)
      ) {
        return "concatAtTokenBoundary";
      }
      sawLiteralDelimiter ||= span.literal.text.length > 0;
      leftLiteral = span.literal.text;
      if (classBoundaryEffectForExpression(span.expression) === "concatAtTokenBoundary") {
        return "concatAtTokenBoundary";
      }
    }
    return sawLiteralDelimiter ? "concatInsideToken" : "unknownBoundary";
  }
  return ts.isCallExpression(unwrapped) ? classBoundaryEffectForCall(unwrapped) : "unknownBoundary";
}

function boundaryEffectForBinaryConcat(
  left: ts.Expression,
  right: ts.Expression,
): FlowBlockSnapshot["boundaryEffect"] {
  const leftEdge = expressionLiteralEdge(left, "trailing");
  const rightEdge = expressionLiteralEdge(right, "leading");
  if (leftEdge === "whitespace" || rightEdge === "whitespace") {
    return "concatAtTokenBoundary";
  }
  return leftEdge === "nonWhitespace" || rightEdge === "nonWhitespace"
    ? "concatInsideToken"
    : "unknownBoundary";
}

function classBoundaryEffectForCall(call: ts.CallExpression): FlowBlockSnapshot["boundaryEffect"] {
  const callee = transparentExpression(call.expression);
  if (
    ts.isIdentifier(callee) &&
    (callee.text === "clsx" || callee.text === "classnames" || callee.text === "classNames")
  ) {
    return "concatAtTokenBoundary";
  }
  if (!ts.isPropertyAccessExpression(callee) || callee.name.text !== "join") {
    return "unknownBoundary";
  }
  const separator = call.arguments[0] ? staticStringValue(call.arguments[0]) : ",";
  if (separator === null) return "unknownBoundary";
  return [...separator].some(isDomClassAsciiWhitespace)
    ? "concatAtTokenBoundary"
    : "concatInsideToken";
}

type LiteralEdge = "empty" | "whitespace" | "nonWhitespace" | "unknown";

function expressionLiteralEdge(
  expression: ts.Expression,
  side: "leading" | "trailing",
): LiteralEdge {
  const unwrapped = transparentExpression(expression);
  const literal = staticStringValue(unwrapped);
  if (literal !== null) return literalEdge(literal, side);
  if (ts.isTemplateExpression(unwrapped)) {
    const text =
      side === "leading" ? unwrapped.head.text : unwrapped.templateSpans.at(-1)?.literal.text;
    return literalEdge(text ?? "", side);
  }
  if (
    ts.isBinaryExpression(unwrapped) &&
    unwrapped.operatorToken.kind === ts.SyntaxKind.PlusToken
  ) {
    const primary = side === "leading" ? unwrapped.left : unwrapped.right;
    const fallback = side === "leading" ? unwrapped.right : unwrapped.left;
    const edge = expressionLiteralEdge(primary, side);
    return edge === "empty" ? expressionLiteralEdge(fallback, side) : edge;
  }
  return "unknown";
}

function literalEdge(value: string, side: "leading" | "trailing"): LiteralEdge {
  if (value.length === 0) return "empty";
  const character = side === "leading" ? [...value][0] : [...value].at(-1);
  return character && isDomClassAsciiWhitespace(character) ? "whitespace" : "nonWhitespace";
}

function transparentExpression(expression: ts.Expression): ts.Expression {
  if (
    ts.isParenthesizedExpression(expression) ||
    ts.isAsExpression(expression) ||
    ts.isTypeAssertionExpression(expression) ||
    ts.isNonNullExpression(expression) ||
    ts.isSatisfiesExpression(expression)
  ) {
    return transparentExpression(expression.expression);
  }
  return expression;
}

function staticStringValue(expression: ts.Expression): string | null {
  const unwrapped = transparentExpression(expression);
  return ts.isStringLiteral(unwrapped) || ts.isNoSubstitutionTemplateLiteral(unwrapped)
    ? unwrapped.text
    : null;
}

function isDomClassAsciiWhitespace(character: string): boolean {
  return (
    character === "\u0009" ||
    character === "\u000a" ||
    character === "\u000c" ||
    character === "\u000d" ||
    character === " "
  );
}

function startsWithDomClassAsciiWhitespace(value: string): boolean {
  const character = [...value][0];
  return character !== undefined && isDomClassAsciiWhitespace(character);
}

function endsWithDomClassAsciiWhitespace(value: string): boolean {
  const character = [...value].at(-1);
  return character !== undefined && isDomClassAsciiWhitespace(character);
}

function shortCircuitExpressionKind(
  expression: ts.BinaryExpression,
): NonNullable<FlowBlockSnapshot["expressionKind"]> {
  switch (expression.operatorToken.kind) {
    case ts.SyntaxKind.AmpersandAmpersandToken:
      return "logicalAnd";
    case ts.SyntaxKind.BarBarToken:
      return "logicalOr";
    case ts.SyntaxKind.QuestionQuestionToken:
      return "nullishCoalesce";
    default:
      throw new Error("not a short-circuit expression");
  }
}

function transferKindForBlockKind(
  kind: FlowBlockSnapshot["kind"],
): FlowBlockSnapshot["transferKind"] {
  switch (kind) {
    case "entry":
      return "entry";
    case "assignment":
      return "assignFacts";
    case "branch":
    case "logicalOperand":
      return "branch";
    case "join":
    case "logicalJoin":
      return "join";
    case "loopHeader":
    case "loopBody":
    case "loopExit":
      return "loop";
    case "break":
      return "break";
    case "terminate":
      return "terminate";
    case "logicalRhs":
      return "assignFacts";
    case "exit":
      return "exit";
    default:
      kind satisfies never;
      return "exit";
  }
}
