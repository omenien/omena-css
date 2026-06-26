import "@typespec/json-schema";

using TypeSpec.JsonSchema;

@jsonSchema
namespace Omena.ParseTree;

model ParseTreeNodeV0 {
  kind: string;
  byteSpan: {
    start: int32;
    end: int32;
  };
  range: {
    start: {
      line: int32;
      character: int32;
    };
    end: {
      line: int32;
      character: int32;
    };
  };
  text?: string;
  children: ParseTreeNodeV0[];
  bogus?: boolean;
  error?: string;
}
