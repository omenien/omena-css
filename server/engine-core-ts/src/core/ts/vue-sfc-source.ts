export function projectVueSfcScriptToTypeScriptSource(content: string): string {
  const projected: string[] = Array.from(content, (char) => (char === "\n" ? "\n" : " "));
  const scriptPattern = /<script\b[^>]*>([\s\S]*?)<\/script>/giu;

  for (const match of content.matchAll(scriptPattern)) {
    const fullMatch = match[0];
    const scriptContent = match[1] ?? "";
    const matchStart = match.index ?? 0;
    const contentStart = matchStart + fullMatch.indexOf(scriptContent);

    for (let index = 0; index < scriptContent.length; index += 1) {
      projected[contentStart + index] = scriptContent[index]!;
    }
  }

  return projected.join("");
}
