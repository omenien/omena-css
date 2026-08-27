export function maskRustCfgTestItems(source: string): string {
  const structure = rustStructuralSource(source);
  const spans: { readonly start: number; readonly end: number }[] = [];
  for (const match of structure.matchAll(/#\s*\[\s*cfg\s*\(/gu)) {
    const openParenthesis = match.index + match[0].lastIndexOf("(");
    const closeParenthesis = matchingDelimiter(structure, openParenthesis, "(", ")");
    if (closeParenthesis === undefined) continue;
    let attributeEnd = closeParenthesis + 1;
    while (/\s/u.test(structure[attributeEnd] ?? "")) attributeEnd += 1;
    if (structure[attributeEnd] !== "]") continue;
    attributeEnd += 1;
    if (!cfgContainsPositiveTest(structure.slice(openParenthesis + 1, closeParenthesis))) continue;
    const end = cfgTestItemEnd(structure, attributeEnd);
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

function cfgContainsPositiveTest(expression: string): boolean {
  const callStack: boolean[] = [];
  let negationDepth = 0;
  let pendingIdentifier: string | undefined;
  for (let index = 0; index < expression.length;) {
    const current = expression[index] ?? "";
    if (/\s/u.test(current)) {
      index += 1;
      continue;
    }
    const identifier = expression.slice(index).match(/^[A-Za-z_][A-Za-z0-9_]*/u)?.[0];
    if (identifier) {
      if (identifier === "test" && negationDepth % 2 === 0) return true;
      pendingIdentifier = identifier;
      index += identifier.length;
      continue;
    }
    if (current === "(") {
      const isNegation = pendingIdentifier === "not";
      callStack.push(isNegation);
      if (isNegation) negationDepth += 1;
    } else if (current === ")") {
      if (callStack.pop()) negationDepth -= 1;
    }
    pendingIdentifier = undefined;
    index += 1;
  }
  return false;
}

export function assertRustCfgTestMaskContract(): void {
  const source = [
    "#[cfg(test)]\nfn direct_test_item() {}",
    '#[cfg(any(test, feature = "test-support"))]\nfn compound_test_item() {}',
    "#[cfg(not(test))]\nfn production_not_test_item() {}",
    '#[cfg(all(feature = "release", not(test)))]\nfn production_compound_not_test_item() {}',
    '#[cfg(not(any(test, feature = "test-support")))]\nfn production_outer_not_test_item() {}',
    "#[cfg(not(not(test)))]\nfn double_negated_test_item() {}",
    "fn adjacent_production_item() {}",
  ].join("\n");
  const masked = maskRustCfgTestItems(source);
  const mustBeMasked = ["direct_test_item", "compound_test_item", "double_negated_test_item"];
  const mustRemain = [
    "production_not_test_item",
    "production_compound_not_test_item",
    "production_outer_not_test_item",
    "adjacent_production_item",
  ];
  for (const name of mustBeMasked) {
    if (masked.includes(name)) throw new Error(`cfg(test) masker retained test-only item ${name}`);
  }
  for (const name of mustRemain) {
    if (!masked.includes(name)) throw new Error(`cfg(test) masker removed production item ${name}`);
  }
}

function matchingDelimiter(
  source: string,
  openOffset: number,
  open: string,
  close: string,
): number | undefined {
  let depth = 0;
  for (let index = openOffset; index < source.length; index += 1) {
    if (source[index] === open) depth += 1;
    else if (source[index] === close) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return undefined;
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
