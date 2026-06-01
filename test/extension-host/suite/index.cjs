const assert = require("node:assert/strict");
const vscode = require("vscode");

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function retry(task, attempts = 20, delayMs = 500, lastError) {
  try {
    return await task();
  } catch (error) {
    if (attempts <= 1) {
      throw lastError ?? error;
    }
    await sleep(delayMs);
    return retry(task, attempts - 1, delayMs, error);
  }
}

function toTargetUri(location) {
  if (!location) return undefined;
  if ("targetUri" in location) return location.targetUri;
  return location.uri;
}

async function run() {
  const extension = vscode.extensions.getExtension("omena.omena-css");
  assert(extension, "extension should be registered");

  await extension.activate();

  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  assert(workspaceFolder, "workspace fixture should be available");

  const sourceUri = vscode.Uri.joinPath(workspaceFolder.uri, "index.ts");
  const document = await vscode.workspace.openTextDocument(sourceUri);
  await vscode.window.showTextDocument(document);

  const source = document.getText();
  const marker = '"root"';
  const markerOffset = source.indexOf(marker);
  assert.notEqual(markerOffset, -1, "fixture should contain the class marker");
  const position = document.positionAt(markerOffset + 2);

  const definitions = await retry(async () => {
    const result = await vscode.commands.executeCommand(
      "vscode.executeDefinitionProvider",
      sourceUri,
      position,
    );

    assert(Array.isArray(result) && result.length > 0, "definition provider returned no results");
    return result;
  });

  const targetUris = definitions.map(toTargetUri).filter(Boolean);
  assert(
    targetUris.some((uri) => uri.fsPath.endsWith("styles.module.scss")),
    "definition provider did not resolve to styles.module.scss",
  );
}

module.exports = { run };
