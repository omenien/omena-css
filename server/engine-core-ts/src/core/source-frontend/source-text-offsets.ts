/**
 * Convert a JavaScript UTF-16 source offset to the UTF-8 byte offset used on Rust wires.
 * An offset inside a surrogate pair floors to that code point's start.
 */
export function utf8ByteOffsetAtUtf16Offset(source: string, utf16Offset: number): number {
  let bytes = 0;
  for (let offset = 0; offset < source.length;) {
    if (offset >= utf16Offset) return bytes;
    const codePoint = source.codePointAt(offset) ?? 0;
    const width = codePoint > 0xffff ? 2 : 1;
    if (offset + width > utf16Offset) return bytes;
    bytes += utf8CodePointByteLength(codePoint);
    offset += width;
  }
  return bytes;
}

/**
 * Convert a Rust UTF-8 byte offset to the JavaScript UTF-16 source convention.
 * An offset inside a multi-byte sequence floors to that code point's start.
 */
export function utf16OffsetAtUtf8ByteOffset(source: string, byteOffset: number): number {
  let bytes = 0;
  for (let offset = 0; offset < source.length;) {
    if (bytes >= byteOffset) return offset;
    const codePoint = source.codePointAt(offset) ?? 0;
    const width = codePoint > 0xffff ? 2 : 1;
    const nextBytes = bytes + utf8CodePointByteLength(codePoint);
    if (nextBytes > byteOffset) return offset;
    bytes = nextBytes;
    offset += width;
  }
  return source.length;
}

function utf8CodePointByteLength(codePoint: number): number {
  if (codePoint <= 0x7f) return 1;
  if (codePoint <= 0x7ff) return 2;
  if (codePoint <= 0xffff) return 3;
  return 4;
}
