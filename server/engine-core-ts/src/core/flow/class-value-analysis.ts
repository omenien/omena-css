import ts from "typescript";
import type { Range } from "@omena/shared";
import {
  concatenateClassValues,
  concatenateWithUnknownLeft,
  concatenateWithUnknownRight,
} from "../abstract-value/class-value-domain";
import { buildFlowSlice } from "./flow-slice";
import {
  exactValue,
  markBranched,
  mergeValues,
  toFlowResolution,
  type ClassValueLattice,
  type FlowResolution,
} from "./lattice";
import type { FlowNode } from "./cfg";

type FlowEnv = Map<string, ClassValueLattice>;

interface FlowState {
  readonly env: FlowEnv;
  readonly terminated: boolean;
  readonly termination: "none" | "function" | "break";
}

export function resolveFlowClassValues(
  sourceFile: ts.SourceFile,
  range: Range,
  variableName: string,
): FlowResolution | null {
  const slice = buildFlowSlice(sourceFile, range, variableName);
  if (!slice) return null;

  const result = analyzeNodes(slice.nodes, new Map(), sourceFile);
  return toFlowResolution(result.env.get(variableName) ?? null);
}

function analyzeNodes(
  nodes: readonly FlowNode[],
  incoming: FlowEnv,
  sourceFile: ts.SourceFile,
): FlowState {
  let env = cloneEnv(incoming);
  let terminated = false;

  for (const node of nodes) {
    if (terminated) break;

    switch (node.kind) {
      case "assignment": {
        const resolved = resolveExpression(node.expression, env, sourceFile);
        if (resolved) {
          env.set(node.variableName, resolved);
        } else {
          env.delete(node.variableName);
        }
        break;
      }
      case "branch": {
        if (node.referenceLocation === "then") {
          return analyzeNodes(node.thenNodes, env, sourceFile);
        }
        if (node.referenceLocation === "else") {
          return analyzeNodes(node.elseNodes, env, sourceFile);
        }

        const thenState = analyzeNodes(node.thenNodes, env, sourceFile);
        const elseState =
          node.elseNodes.length > 0
            ? analyzeNodes(node.elseNodes, env, sourceFile)
            : { env, terminated: false, termination: "none" as const };
        const merged = mergeEnvs(env, thenState, elseState);
        env = merged.env;
        terminated = merged.terminated;
        break;
      }
      case "loop": {
        if (node.referenceLocation === "body") {
          return analyzeNodes(node.bodyNodes, env, sourceFile);
        }

        const bodyState = analyzeNodes(node.bodyNodes, env, sourceFile);
        env = mergeLoopEnv(env, bodyState);
        break;
      }
      case "break":
        terminated = true;
        return { env, terminated, termination: "break" };
      case "terminate":
        terminated = true;
        return { env, terminated, termination: "function" };
      default:
        node satisfies never;
        break;
    }
  }

  return { env, terminated, termination: "none" };
}

function resolveExpression(
  expression: ts.Expression | null,
  env: FlowEnv,
  sourceFile: ts.SourceFile,
): ClassValueLattice | null {
  if (!expression) return null;

  if (ts.isStringLiteral(expression) || ts.isNoSubstitutionTemplateLiteral(expression)) {
    return exactValue(expression.text);
  }

  if (ts.isIdentifier(expression)) {
    return env.get(expression.text) ?? null;
  }

  if (ts.isParenthesizedExpression(expression)) {
    return resolveExpression(expression.expression, env, sourceFile);
  }

  if (ts.isAsExpression(expression) || ts.isTypeAssertionExpression(expression)) {
    return resolveExpression(expression.expression, env, sourceFile);
  }

  if (ts.isConditionalExpression(expression)) {
    const whenTrue = resolveExpression(expression.whenTrue, env, sourceFile);
    const whenFalse = resolveExpression(expression.whenFalse, env, sourceFile);
    return markBranched(mergeValues(whenTrue, whenFalse));
  }

  if (
    ts.isBinaryExpression(expression) &&
    expression.operatorToken.kind === ts.SyntaxKind.PlusToken
  ) {
    const left = resolveExpression(expression.left, env, sourceFile);
    const right = resolveExpression(expression.right, env, sourceFile);

    if (left && right) {
      return {
        abstractValue: concatenateClassValues(left.abstractValue, right.abstractValue),
        reason:
          left.reason === "flowBranch" || right.reason === "flowBranch"
            ? "flowBranch"
            : "flowLiteral",
      };
    }

    if (left) {
      return {
        abstractValue: concatenateWithUnknownRight(left.abstractValue),
        reason: left.reason,
      };
    }

    if (right) {
      return {
        abstractValue: concatenateWithUnknownLeft(right.abstractValue),
        reason: right.reason,
      };
    }
  }

  if (
    ts.isBinaryExpression(expression) &&
    (expression.operatorToken.kind === ts.SyntaxKind.AmpersandAmpersandToken ||
      expression.operatorToken.kind === ts.SyntaxKind.BarBarToken ||
      expression.operatorToken.kind === ts.SyntaxKind.QuestionQuestionToken)
  ) {
    return resolveShortCircuitExpression(expression, env, sourceFile);
  }

  if (ts.isCallExpression(expression)) {
    return resolveDirectFunctionCall(expression, env, sourceFile);
  }

  return null;
}

function resolveShortCircuitExpression(
  expression: ts.BinaryExpression,
  env: FlowEnv,
  sourceFile: ts.SourceFile,
): ClassValueLattice | null {
  const left = resolveExpression(expression.left, env, sourceFile);
  const right = resolveExpression(expression.right, env, sourceFile);

  if (expression.operatorToken.kind === ts.SyntaxKind.AmpersandAmpersandToken) {
    if (left && isDefinitelyTruthyClassValue(left)) return right;
    return markBranched(right);
  }

  if (left && isDefinitelyTruthyClassValue(left)) return left;
  return markBranched(mergeValues(left, right));
}

function isDefinitelyTruthyClassValue(value: ClassValueLattice): boolean {
  const abstractValue = value.abstractValue;
  return abstractValue.kind === "exact" && abstractValue.value.length > 0;
}

interface ReturnAnalysis {
  readonly value: ClassValueLattice | null;
  readonly complete: boolean;
  readonly valid: boolean;
}

function resolveDirectFunctionCall(
  expression: ts.CallExpression,
  env: FlowEnv,
  sourceFile: ts.SourceFile,
): ClassValueLattice | null {
  if (!ts.isIdentifier(expression.expression)) return null;
  const callable = findSameFileFunction(sourceFile, expression.expression.text);
  if (!callable?.body || !ts.isBlock(callable.body)) return null;

  const analysis = analyzeReturnStatements(callable.body.statements, env, sourceFile, callable);
  if (!analysis.complete || !analysis.valid) return null;
  return analysis.value;
}

function analyzeReturnStatements(
  statements: readonly ts.Statement[],
  env: FlowEnv,
  sourceFile: ts.SourceFile,
  callable?: ts.FunctionLikeDeclaration,
): ReturnAnalysis {
  let value: ClassValueLattice | null = null;

  for (const statement of statements) {
    const analysis = analyzeReturnStatement(statement, env, sourceFile, callable);
    if (!analysis.valid) return analysis;
    value = mergeValues(value, analysis.value);
    if (analysis.complete) {
      return { value, complete: true, valid: true };
    }
  }

  return { value, complete: false, valid: true };
}

function analyzeReturnStatement(
  statement: ts.Statement,
  env: FlowEnv,
  sourceFile: ts.SourceFile,
  callable?: ts.FunctionLikeDeclaration,
): ReturnAnalysis {
  if (ts.isReturnStatement(statement)) {
    const value = resolveExpression(statement.expression ?? null, env, sourceFile);
    return {
      value,
      complete: true,
      valid: value !== null,
    };
  }

  if (ts.isBlock(statement)) {
    return analyzeReturnStatements(statement.statements, env, sourceFile, callable);
  }

  if (ts.isIfStatement(statement)) {
    const thenAnalysis = analyzeReturnStatement(statement.thenStatement, env, sourceFile, callable);
    const elseAnalysis = statement.elseStatement
      ? analyzeReturnStatement(statement.elseStatement, env, sourceFile, callable)
      : { value: null, complete: false, valid: true };

    return {
      value: mergeValues(thenAnalysis.value, elseAnalysis.value),
      complete: thenAnalysis.complete && elseAnalysis.complete,
      valid: thenAnalysis.valid && elseAnalysis.valid,
    };
  }

  if (ts.isSwitchStatement(statement)) {
    let value: ClassValueLattice | null = null;
    let hasDefault = false;

    for (const clause of statement.caseBlock.clauses) {
      if (ts.isDefaultClause(clause)) hasDefault = true;
      const analysis = analyzeReturnStatements(clause.statements, env, sourceFile, callable);
      if (!analysis.valid) return analysis;
      value = mergeValues(value, analysis.value);
      if (!analysis.complete) {
        return { value, complete: false, valid: true };
      }
    }

    return {
      value,
      complete:
        hasDefault ||
        (callable ? isExhaustiveStringSwitch(statement, callable, sourceFile) : false),
      valid: true,
    };
  }

  return { value: null, complete: false, valid: true };
}

function findSameFileFunction(
  sourceFile: ts.SourceFile,
  name: string,
): ts.FunctionLikeDeclaration | null {
  for (const statement of sourceFile.statements) {
    if (ts.isFunctionDeclaration(statement) && statement.name?.text === name) {
      return statement;
    }

    if (!ts.isVariableStatement(statement)) continue;

    for (const declaration of statement.declarationList.declarations) {
      if (!ts.isIdentifier(declaration.name) || declaration.name.text !== name) continue;
      if (
        declaration.initializer &&
        (ts.isArrowFunction(declaration.initializer) ||
          ts.isFunctionExpression(declaration.initializer))
      ) {
        return declaration.initializer;
      }
    }
  }

  return null;
}

function isExhaustiveStringSwitch(
  statement: ts.SwitchStatement,
  callable: ts.FunctionLikeDeclaration,
  sourceFile: ts.SourceFile,
): boolean {
  if (!ts.isIdentifier(statement.expression)) return false;
  const discriminantName = statement.expression.text;

  const parameter = callable.parameters.find(
    (candidate) => ts.isIdentifier(candidate.name) && candidate.name.text === discriminantName,
  );
  if (!parameter?.type) return false;

  const parameterMembers = resolveStringLiteralTypeMembers(parameter.type, sourceFile, new Set());
  if (!parameterMembers || parameterMembers.length === 0) return false;

  const caseMembers = statement.caseBlock.clauses.flatMap((clause) => {
    if (!ts.isCaseClause(clause)) return [];
    return ts.isStringLiteral(clause.expression) ? [clause.expression.text] : [];
  });
  if (caseMembers.length !== statement.caseBlock.clauses.length) return false;

  const expected = [...new Set(parameterMembers)].toSorted();
  const actual = [...new Set(caseMembers)].toSorted();
  return (
    expected.length === actual.length && expected.every((value, index) => value === actual[index])
  );
}

function resolveStringLiteralTypeMembers(
  typeNode: ts.TypeNode,
  sourceFile: ts.SourceFile,
  seen: Set<string>,
): readonly string[] | null {
  if (ts.isUnionTypeNode(typeNode)) {
    const members: string[] = [];
    for (const member of typeNode.types) {
      if (!ts.isLiteralTypeNode(member) || !ts.isStringLiteral(member.literal)) {
        return null;
      }
      members.push(member.literal.text);
    }
    return members;
  }

  if (ts.isParenthesizedTypeNode(typeNode)) {
    return resolveStringLiteralTypeMembers(typeNode.type, sourceFile, seen);
  }

  if (ts.isTypeReferenceNode(typeNode) && ts.isIdentifier(typeNode.typeName)) {
    const name = typeNode.typeName.text;
    if (seen.has(name)) return null;
    seen.add(name);
    const alias = findTypeAliasDeclaration(sourceFile, name);
    return alias?.type ? resolveStringLiteralTypeMembers(alias.type, sourceFile, seen) : null;
  }

  return null;
}

function findTypeAliasDeclaration(
  sourceFile: ts.SourceFile,
  name: string,
): ts.TypeAliasDeclaration | null {
  for (const statement of sourceFile.statements) {
    if (ts.isTypeAliasDeclaration(statement) && statement.name.text === name) {
      return statement;
    }
  }
  return null;
}

function mergeEnvs(base: FlowEnv, left: FlowState, right: FlowState): FlowState {
  if (left.termination === "function" && right.termination === "function") {
    return { env: new Map(), terminated: true, termination: "function" };
  }
  if (left.termination === "function") {
    return {
      env: cloneEnv(right.env),
      terminated: right.terminated,
      termination: right.termination,
    };
  }
  if (right.termination === "function") {
    return { env: cloneEnv(left.env), terminated: left.terminated, termination: left.termination };
  }

  const merged = cloneEnv(base);
  const keys = new Set([...left.env.keys(), ...right.env.keys(), ...base.keys()]);
  for (const key of keys) {
    const value = mergeValues(left.env.get(key) ?? null, right.env.get(key) ?? null);
    if (value) {
      merged.set(key, value);
    } else {
      merged.delete(key);
    }
  }
  const breakTerminated = left.termination === "break" && right.termination === "break";
  return {
    env: merged,
    terminated: breakTerminated,
    termination: breakTerminated ? "break" : "none",
  };
}

function mergeLoopEnv(base: FlowEnv, bodyState: FlowState): FlowEnv {
  if (bodyState.termination === "function") return cloneEnv(base);

  const merged = cloneEnv(base);
  const keys = new Set([...bodyState.env.keys(), ...base.keys()]);
  for (const key of keys) {
    const value = mergeValues(base.get(key) ?? null, bodyState.env.get(key) ?? null);
    if (value) {
      merged.set(key, value);
    } else {
      merged.delete(key);
    }
  }
  return merged;
}

function cloneEnv(env: FlowEnv): FlowEnv {
  return new Map(Array.from(env.entries(), ([key, value]) => [key, { ...value }]));
}
