export function maskRustCfgTestItems(source: string): string {
  const spans: { readonly start: number; readonly end: number }[] = [];
  const testAttribute = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/gu;
  for (const match of source.matchAll(testAttribute)) {
    const end = cfgTestItemEnd(source, match.index + match[0].length);
    if (end !== undefined) spans.push({ start: match.index, end });
  }
  if (spans.length === 0) return source;

  const chars = source.split("");
  for (const span of spans) {
    for (let index = span.start; index < span.end; index += 1) {
      if (chars[index] !== "\n") chars[index] = " ";
    }
  }
  return chars.join("");
}

function cfgTestItemEnd(source: string, attributeEnd: number): number | undefined {
  let cursor = attributeEnd;
  while (cursor < source.length) {
    while (/\s/u.test(source[cursor] ?? "")) cursor += 1;
    if (!source.startsWith("#", cursor)) break;
    const attributeClose = source.indexOf("]", cursor + 1);
    if (attributeClose < 0) return undefined;
    cursor = attributeClose + 1;
  }

  let parentheses = 0;
  let brackets = 0;
  for (let index = cursor; index < source.length; index += 1) {
    const current = source[index];
    if (current === "(") parentheses += 1;
    else if (current === ")") parentheses -= 1;
    else if (current === "[") brackets += 1;
    else if (current === "]") brackets -= 1;
    else if (current === "{" && parentheses === 0 && brackets === 0) {
      const closeBrace = matchingBrace(source, index);
      return closeBrace === undefined ? undefined : closeBrace + 1;
    } else if (current === ";" && parentheses === 0 && brackets === 0) {
      return index + 1;
    }
  }
  return undefined;
}

function matchingBrace(source: string, openBrace: number): number | undefined {
  let depth = 0;
  for (let index = openBrace; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    else if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return undefined;
}
