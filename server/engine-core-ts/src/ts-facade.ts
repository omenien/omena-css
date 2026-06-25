import ts from "typescript";

/**
 * TS7 Surface-1 migration seam.
 *
 * Keep classic TypeScript compiler API access behind this module. The current
 * body intentionally delegates to TypeScript 6; future AST migration should
 * change this seam instead of each source analyzer call-site.
 *
 * See ./ts-facade-migration-map.md for the pending TS6 to TS7 surface mapping.
 */
// oxlint-disable-next-line import/no-default-export -- Preserve existing compiler API call-site shape behind the seam.
export default ts;

export function nodeStart(node: ts.Node, sourceFile?: ts.SourceFile): number {
  return node.getStart(sourceFile);
}

export function nodeEnd(node: ts.Node): number {
  return node.end;
}

export function nodeText(node: ts.Node, sourceFile?: ts.SourceFile): string {
  return node.getText(sourceFile);
}

export function lineCharOfPosition(
  sourceFile: ts.SourceFile,
  position: number,
): ts.LineAndCharacter {
  return ts.getLineAndCharacterOfPosition(sourceFile, position);
}

export function positionOfLineChar(
  sourceFile: ts.SourceFile,
  lineChar: ts.LineAndCharacter,
): number {
  return ts.getPositionOfLineAndCharacter(sourceFile, lineChar.line, lineChar.character);
}
