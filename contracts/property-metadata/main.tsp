import "@typespec/json-schema";

using TypeSpec.JsonSchema;

@jsonSchema
namespace Omena.PropertyMetadata;

model CssPropertyMetadataV1 {
  schemaVersion: string;
  product: string;
  source: CssPropertyMetadataSourceV1;
  customPropertyPolicy: CssCustomPropertyPolicyV1;
  properties: CssPropertyMetadataRecordV1[];
}

model CssPropertyMetadataSourceV1 {
  package: string;
  version: string;
  sourcePath: string;
  tool: string;
}

model CssCustomPropertyPolicyV1 {
  inherited: boolean;
  initialValue: string;
}

model CssPropertyMetadataRecordV1 {
  propertyId: string;
  canonicalName: string;
  inherited: boolean;
  initialValue: string;
}
