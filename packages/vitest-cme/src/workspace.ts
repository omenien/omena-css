export interface Position {
  readonly line: number;
  readonly character: number;
}

export interface Range {
  readonly start: Position;
  readonly end: Position;
}

export interface CmeMarker {
  readonly name: string;
  readonly filePath: string;
  readonly position: Position;
}

export interface CmeRangeMarker {
  readonly name: string;
  readonly filePath: string;
  readonly range: Range;
}

export interface CmeParsedFile {
  readonly filePath: string;
  readonly content: string;
  readonly markers: ReadonlyMap<string, CmeMarker>;
  readonly ranges: ReadonlyMap<string, CmeRangeMarker>;
}

export interface CmeWorkspace {
  readonly files: ReadonlyMap<string, CmeParsedFile>;
  readonly filePaths: readonly string[];
  file(filePath: string): CmeParsedFile;
  marker(name?: string, filePath?: string): CmeMarker;
  range(name: string, filePath?: string): CmeRangeMarker;
}

interface MutableParseState {
  readonly filePath: string;
  output: string;
  position: Position;
  readonly markers: Map<string, CmeMarker>;
  readonly ranges: Map<string, CmeRangeMarker>;
  readonly openRanges: Map<string, Position>;
}

const MARKER_NAME = "[A-Za-z_][A-Za-z0-9_-]*";
const AT_MARKER = new RegExp(String.raw`^\/\*at:(${MARKER_NAME})\*\/`);
const RANGE_START_MARKER = new RegExp(String.raw`^\/\*<(${MARKER_NAME})>\*\/`);
const RANGE_END_MARKER = new RegExp(String.raw`^\/\*<\/(${MARKER_NAME})>\*\/`);

export class MarkerParseError extends Error {
  constructor(
    message: string,
    readonly filePath: string,
    readonly position: Position,
  ) {
    super(`${filePath}:${position.line + 1}:${position.character + 1}: ${message}`);
    this.name = "MarkerParseError";
  }
}

export function workspace(files: Record<string, string>): CmeWorkspace {
  const parsedFiles = new Map(
    Object.entries(files).map(([filePath, content]) => [filePath, parseFile(filePath, content)]),
  );

  return {
    files: parsedFiles,
    filePaths: [...parsedFiles.keys()].toSorted(),
    file(filePath) {
      const file = parsedFiles.get(filePath);
      if (!file) {
        throw new Error(
          `Unknown fixture file "${filePath}". Known files: ${formatKnown(parsedFiles.keys())}`,
        );
      }
      return file;
    },
    marker(name = "cursor", filePath) {
      return findNamedEntry(parsedFiles, "marker", name, filePath, (file) => file.markers);
    },
    range(name, filePath) {
      return findNamedEntry(parsedFiles, "range", name, filePath, (file) => file.ranges);
    },
  };
}

function parseFile(filePath: string, input: string): CmeParsedFile {
  const state: MutableParseState = {
    filePath,
    output: "",
    position: { line: 0, character: 0 },
    markers: new Map(),
    ranges: new Map(),
    openRanges: new Map(),
  };

  for (let index = 0; index < input.length;) {
    const rest = input.slice(index);

    if (rest.startsWith(String.raw`/*\|*/`)) {
      appendText(state, "/*|*/");
      index += String.raw`/*\|*/`.length;
      continue;
    }

    if (rest.startsWith("/*|*/")) {
      addMarker(state, "cursor");
      index += "/*|*/".length;
      continue;
    }

    const atMarker = AT_MARKER.exec(rest);
    if (atMarker) {
      addMarker(state, atMarker[1]!);
      index += atMarker[0].length;
      continue;
    }

    const rangeStart = RANGE_START_MARKER.exec(rest);
    if (rangeStart) {
      openRange(state, rangeStart[1]!);
      index += rangeStart[0].length;
      continue;
    }

    const rangeEnd = RANGE_END_MARKER.exec(rest);
    if (rangeEnd) {
      closeRange(state, rangeEnd[1]!);
      index += rangeEnd[0].length;
      continue;
    }

    if (rest.startsWith("/*<") || rest.startsWith("/*</")) {
      throw new MarkerParseError("Malformed range marker.", filePath, state.position);
    }

    appendText(state, input[index]!);
    index += 1;
  }

  for (const [name, start] of state.openRanges) {
    throw new MarkerParseError(
      `Range marker "${name}" was opened but not closed.`,
      filePath,
      start,
    );
  }

  return {
    filePath,
    content: state.output,
    markers: state.markers,
    ranges: state.ranges,
  };
}

function addMarker(state: MutableParseState, name: string): void {
  if (state.markers.has(name)) {
    throw new MarkerParseError(`Duplicate marker "${name}".`, state.filePath, state.position);
  }
  state.markers.set(name, {
    name,
    filePath: state.filePath,
    position: state.position,
  });
}

function openRange(state: MutableParseState, name: string): void {
  if (state.openRanges.has(name) || state.ranges.has(name)) {
    throw new MarkerParseError(`Duplicate range marker "${name}".`, state.filePath, state.position);
  }
  state.openRanges.set(name, state.position);
}

function closeRange(state: MutableParseState, name: string): void {
  const start = state.openRanges.get(name);
  if (!start) {
    throw new MarkerParseError(
      `Range marker "${name}" was closed without an opening marker.`,
      state.filePath,
      state.position,
    );
  }
  state.openRanges.delete(name);
  state.ranges.set(name, {
    name,
    filePath: state.filePath,
    range: {
      start,
      end: state.position,
    },
  });
}

function appendText(state: MutableParseState, text: string): void {
  state.output += text;
  for (const char of text) {
    state.position =
      char === "\n"
        ? { line: state.position.line + 1, character: 0 }
        : { line: state.position.line, character: state.position.character + 1 };
  }
}

function findNamedEntry<T>(
  files: ReadonlyMap<string, CmeParsedFile>,
  kind: "marker" | "range",
  name: string,
  filePath: string | undefined,
  select: (file: CmeParsedFile) => ReadonlyMap<string, T>,
): T {
  if (filePath) {
    const file = files.get(filePath);
    if (!file) {
      throw new Error(
        `Unknown fixture file "${filePath}". Known files: ${formatKnown(files.keys())}`,
      );
    }
    const entry = select(file).get(name);
    if (!entry) {
      throw new Error(`Missing ${kind} "${name}" in "${filePath}".`);
    }
    return entry;
  }

  const matches = [...files.values()].flatMap((file) => {
    const entry = select(file).get(name);
    return entry ? [entry] : [];
  });
  if (matches.length === 0) {
    throw new Error(`Missing ${kind} "${name}".`);
  }
  if (matches.length > 1) {
    throw new Error(`Ambiguous ${kind} "${name}". Pass a filePath.`);
  }
  return matches[0]!;
}

function formatKnown(values: Iterable<string>): string {
  return [...values].toSorted().join(", ");
}
