import type { BinderDecl, BinderResolution, BinderScope, SourceBinderResult } from "./scope-types";

export function findInnermostScopeAtOffset(
  binder: SourceBinderResult,
  offset: number,
): BinderScope | null {
  let winner: BinderScope | null = null;
  for (const scope of binder.scopes) {
    if (offset < scope.span.start || offset > scope.span.end) continue;
    if (!winner) {
      winner = scope;
      continue;
    }
    const winnerSize = winner.span.end - winner.span.start;
    const scopeSize = scope.span.end - scope.span.start;
    if (scopeSize <= winnerSize) {
      winner = scope;
    }
  }
  return winner;
}

export function resolveIdentifierAtOffset(
  binder: SourceBinderResult,
  name: string,
  offset: number,
): BinderResolution | null {
  const scope = findInnermostScopeAtOffset(binder, offset);
  if (!scope) return null;

  let currentScopeId: string | undefined = scope.id;
  let depth = 0;
  while (currentScopeId) {
    const match = findVisibleDeclInScope(binder, currentScopeId, name, offset);
    if (match) {
      return { refId: `offset:${offset}:${name}`, declId: match.id, depth };
    }
    currentScopeId = binder.scopes.find((entry) => entry.id === currentScopeId)?.parentScopeId;
    depth += 1;
  }
  return null;
}

export function getDeclById(binder: SourceBinderResult, declId: string): BinderDecl | null {
  return binder.decls.find((decl) => decl.id === declId) ?? null;
}

function findVisibleDeclInScope(
  binder: SourceBinderResult,
  scopeId: string,
  name: string,
  offset: number,
): BinderDecl | null {
  const candidates = binder.decls.filter(
    (decl) => decl.scopeId === scopeId && decl.name === name && decl.span.start <= offset,
  );
  if (candidates.length === 0) return null;
  return candidates.reduce((best, current) =>
    current.span.start >= best.span.start ? current : best,
  );
}
