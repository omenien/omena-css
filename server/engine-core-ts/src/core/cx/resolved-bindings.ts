import { resolveIdentifierAtOffset } from "../binder/source-binder";
import { findImportDeclId } from "../binder/import-decls";
import type { SourceBinderResult } from "../binder/scope-types";
import type { CxBinding } from "./cx-types";

export interface ResolvedCxBinding {
  readonly cxVarName: string;
  readonly stylesVarName: string;
  readonly scssModulePath: string;
  readonly classNamesImportName: string;
  readonly bindingDeclId: string;
}

export function resolveCxBindings(
  bindings: readonly CxBinding[],
  sourceBinder?: SourceBinderResult,
): readonly ResolvedCxBinding[] {
  return bindings.flatMap((binding, index) => {
    if (!isValidImportedBinding(binding, sourceBinder)) {
      return [];
    }
    return [
      {
        cxVarName: binding.cxVarName,
        stylesVarName: binding.stylesVarName,
        scssModulePath: binding.scssModulePath,
        classNamesImportName: binding.classNamesImportName,
        bindingDeclId: resolveBindingDeclId(binding, sourceBinder, index),
      },
    ];
  });
}

function resolveBindingDeclId(
  binding: CxBinding,
  sourceBinder: SourceBinderResult | undefined,
  index: number,
): string {
  if (!sourceBinder || binding.bindingDeclOffset === undefined) {
    return `synthetic-binding-decl:${index}`;
  }

  const resolution = resolveIdentifierAtOffset(
    sourceBinder,
    binding.cxVarName,
    binding.bindingDeclOffset,
  );
  return resolution?.declId ?? `synthetic-binding-decl:${index}`;
}

function isValidImportedBinding(
  binding: CxBinding,
  sourceBinder: SourceBinderResult | undefined,
): boolean {
  if (
    !sourceBinder ||
    binding.classNamesReferenceOffset === undefined ||
    binding.stylesReferenceOffset === undefined
  ) {
    return true;
  }

  const expectedClassNamesDeclId = findImportDeclId(
    sourceBinder,
    binding.classNamesImportName,
    new Set(["classnames/bind"]),
  );
  const expectedStylesDeclId = findImportDeclId(sourceBinder, binding.stylesVarName);
  if (!expectedClassNamesDeclId || !expectedStylesDeclId) {
    return false;
  }

  const classNamesResolution = resolveIdentifierAtOffset(
    sourceBinder,
    binding.classNamesImportName,
    binding.classNamesReferenceOffset,
  );
  const stylesResolution = resolveIdentifierAtOffset(
    sourceBinder,
    binding.stylesVarName,
    binding.stylesReferenceOffset,
  );

  return (
    classNamesResolution?.declId === expectedClassNamesDeclId &&
    stylesResolution?.declId === expectedStylesDeclId
  );
}
