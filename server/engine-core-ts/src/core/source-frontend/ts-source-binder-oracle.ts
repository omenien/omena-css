import ts, { nodeEnd, nodeStart } from "../../ts-facade";
import type {
  BinderDecl,
  BinderDeclKind,
  BinderScope,
  SourceBinderResult,
  TextSpan,
} from "../binder/scope-types";

export function buildSourceBinder(sourceFile: ts.SourceFile): SourceBinderResult {
  const scopes: BinderScope[] = [];
  const decls: BinderDecl[] = [];
  let nextScopeId = 0;
  let nextDeclId = 0;

  const addScope = (
    kind: BinderScope["kind"],
    node: ts.Node,
    parentScopeId?: string,
  ): BinderScope => {
    const scope: BinderScope = {
      id: `scope:${nextScopeId++}`,
      kind,
      filePath: sourceFile.fileName,
      span: spanOfNode(node, sourceFile),
    };
    if (parentScopeId) {
      Object.assign(scope, { parentScopeId });
    }
    scopes.push(scope);
    return scope;
  };

  const addDecl = (
    kind: BinderDeclKind,
    scopeId: string,
    name: string,
    node: ts.Node,
    importPath?: string,
  ): void => {
    const decl = {
      id: `decl:${nextDeclId++}`,
      kind,
      scopeId,
      name,
      filePath: sourceFile.fileName,
      span: spanOfNode(node, sourceFile),
      ...(importPath ? { importPath } : {}),
    };
    decls.push(decl);
  };

  const visitNode = (node: ts.Node, scopeId: string): void => {
    if (ts.isImportDeclaration(node)) {
      collectImportDecls(node, scopeId, addDecl);
      return;
    }

    if (ts.isFunctionDeclaration(node) && node.name) {
      addDecl("localVar", scopeId, node.name.text, node.name);
    }

    if (
      ts.isFunctionDeclaration(node) ||
      ts.isFunctionExpression(node) ||
      ts.isArrowFunction(node) ||
      ts.isMethodDeclaration(node)
    ) {
      const functionScope = addScope("function", node, scopeId);
      for (const parameter of node.parameters) {
        collectBindingNames(parameter.name, (nameNode) => {
          addDecl("parameter", functionScope.id, nameNode.text, nameNode);
        });
        if (parameter.initializer) {
          visitNode(parameter.initializer, functionScope.id);
        }
      }
      if (node.body) {
        visitNode(node.body, functionScope.id);
      }
      return;
    }

    if (ts.isBlock(node)) {
      const blockScope = addScope("block", node, scopeId);
      for (const statement of node.statements) {
        visitNode(statement, blockScope.id);
      }
      return;
    }

    if (ts.isVariableDeclaration(node)) {
      collectBindingNames(node.name, (nameNode) => {
        addDecl("localVar", scopeId, nameNode.text, nameNode);
      });
    }

    if (ts.isCatchClause(node)) {
      const catchScope = addScope("block", node.block, scopeId);
      if (node.variableDeclaration) {
        collectBindingNames(node.variableDeclaration.name, (nameNode) => {
          addDecl("localVar", catchScope.id, nameNode.text, nameNode);
        });
      }
      for (const statement of node.block.statements) {
        visitNode(statement, catchScope.id);
      }
      return;
    }

    ts.forEachChild(node, (child) => visitNode(child, scopeId));
  };

  const rootScope = addScope("sourceFile", sourceFile);
  for (const statement of sourceFile.statements) {
    visitNode(statement, rootScope.id);
  }

  return {
    filePath: sourceFile.fileName,
    scopes,
    decls: decls.toSorted((left, right) => left.span.start - right.span.start),
  };
}

function collectImportDecls(
  node: ts.ImportDeclaration,
  scopeId: string,
  addDecl: (
    kind: BinderDeclKind,
    scopeId: string,
    name: string,
    node: ts.Node,
    importPath?: string,
  ) => void,
): void {
  if (!ts.isStringLiteral(node.moduleSpecifier)) return;
  const importPath = node.moduleSpecifier.text;
  const clause = node.importClause;
  if (!clause) return;

  if (clause.name) {
    addDecl("import", scopeId, clause.name.text, clause.name, importPath);
  }

  const namedBindings = clause.namedBindings;
  if (!namedBindings) return;

  if (ts.isNamespaceImport(namedBindings)) {
    addDecl("import", scopeId, namedBindings.name.text, namedBindings.name, importPath);
    return;
  }

  for (const element of namedBindings.elements) {
    addDecl("import", scopeId, element.name.text, element.name, importPath);
  }
}

function collectBindingNames(
  name: ts.BindingName,
  onIdentifier: (node: ts.Identifier) => void,
): void {
  if (ts.isIdentifier(name)) {
    onIdentifier(name);
    return;
  }
  for (const element of name.elements) {
    if (ts.isOmittedExpression(element)) continue;
    collectBindingNames(element.name, onIdentifier);
  }
}

function spanOfNode(node: ts.Node, sourceFile: ts.SourceFile): TextSpan {
  return {
    start: nodeStart(node, sourceFile),
    end: nodeEnd(node),
  };
}
