export function maskRustCfgTestItems(source: string): string {
  const structure = rustStructuralSource(source);
  const spans: { readonly start: number; readonly end: number }[] = [];
  const testAttribute = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/gu;
  for (const match of structure.matchAll(testAttribute)) {
    const end = cfgTestItemEnd(structure, match.index + match[0].length);
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

function rustStructuralSource(source: string): string {
  const chars = source.split("");
  const mask = (start: number, end: number) => {
    for (let index = start; index < end; index += 1) {
      if (chars[index] !== "\n") chars[index] = " ";
    }
  };
  let index = 0;
  while (index < source.length) {
    if (source.startsWith("//", index)) {
      const end = source.indexOf("\n", index + 2);
      const commentEnd = end < 0 ? source.length : end;
      mask(index, commentEnd);
      index = commentEnd;
      continue;
    }
    if (source.startsWith("/*", index)) {
      let cursor = index + 2;
      let depth = 1;
      while (cursor < source.length && depth > 0) {
        if (source.startsWith("/*", cursor)) {
          depth += 1;
          cursor += 2;
        } else if (source.startsWith("*/", cursor)) {
          depth -= 1;
          cursor += 2;
        } else {
          cursor += 1;
        }
      }
      mask(index, cursor);
      index = cursor;
      continue;
    }
    const rawPrefix = source.slice(index).match(/^(?:br|r)(?<hashes>#+)?"/u);
    if (rawPrefix?.groups !== undefined) {
      const hashes = rawPrefix.groups.hashes ?? "";
      const close = `"${hashes}`;
      const contentStart = index + rawPrefix[0].length;
      const closeAt = source.indexOf(close, contentStart);
      const end = closeAt < 0 ? source.length : closeAt + close.length;
      mask(index, end);
      index = end;
      continue;
    }
    if (source[index] === '"') {
      let cursor = index + 1;
      while (cursor < source.length) {
        if (source[cursor] === "\\") cursor += 2;
        else if (source[cursor] === '"') {
          cursor += 1;
          break;
        } else cursor += 1;
      }
      mask(index, cursor);
      index = cursor;
      continue;
    }
    if (source[index] === "'") {
      let cursor = index + 1;
      if (source[cursor] === "\\") cursor += 2;
      else
        cursor +=
          source.codePointAt(cursor) !== undefined && source.codePointAt(cursor)! > 0xffff ? 2 : 1;
      if (source[cursor] === "'") {
        cursor += 1;
        mask(index, cursor);
        index = cursor;
        continue;
      }
    }
    index += 1;
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
