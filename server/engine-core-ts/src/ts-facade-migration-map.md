# TS Facade Migration Map

`ts-facade.ts` is the single Surface-1 seam for classic TypeScript JS AST API access.
The current implementation delegates to TypeScript 6. Future TS7 AST migration work
should update the facade and shims instead of editing source analyzer call-sites.

## Guard Renames

| Current TS6 surface            | TS7 migration target                     |
| ------------------------------ | ---------------------------------------- |
| `ts.isParameter`               | `ts.isParameterDeclaration`              |
| `ts.isTypeAssertionExpression` | `ts.isTypeAssertion`                     |
| `ts.isSourceFile`              | `node.kind === ts.SyntaxKind.SourceFile` |

## Shim Body Swaps

| Facade shim                                | Current body                                             | TS7 migration body                  |
| ------------------------------------------ | -------------------------------------------------------- | ----------------------------------- |
| `nodeStart(node, sourceFile)`              | `node.getStart(sourceFile)`                              | `node.pos` plus trivia skipping     |
| `nodeEnd(node)`                            | `node.end`                                               | `node.end`                          |
| `nodeText(node, sourceFile)`               | `node.getText(sourceFile)`                               | `sourceFile.text.slice(start, end)` |
| `lineCharOfPosition(sourceFile, position)` | `ts.getLineAndCharacterOfPosition(sourceFile, position)` | line-start table lookup             |
| `positionOfLineChar(sourceFile, lineChar)` | `ts.getPositionOfLineAndCharacter(...)`                  | line-start table lookup             |
