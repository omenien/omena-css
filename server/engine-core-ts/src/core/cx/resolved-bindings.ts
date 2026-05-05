import ts from "typescript";
import {
  findImportBindingDeclId,
  resolveBindingAtOffset,
  type SourceBindingGraph,
} from "../binder/source-binding-graph";
import type { SourceBinderResult } from "../binder/scope-types";
import type { CxBinding } from "./cx-types";
import { buildSourceBindingGraph } from "../binder/source-binding-graph";

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
  sourceFile?: ts.SourceFile,
): readonly ResolvedCxBinding[] {
  const sourceBindingGraph =
    sourceBinder && sourceFile
      ? buildSourceBindingGraph(
          {
            filePath: sourceFile.fileName,
            kind: "source",
            language: "unknown",
            styleImports: [],
            utilityBindings: [],
            classExpressions: [],
            domainClassReferences: [],
          },
          sourceBinder,
        )
      : undefined;
  return bindings.flatMap((binding, index) => {
    if (!isValidImportedBinding(binding, sourceBindingGraph)) {
      return [];
    }
    return [
      {
        cxVarName: binding.cxVarName,
        stylesVarName: binding.stylesVarName,
        scssModulePath: binding.scssModulePath,
        classNamesImportName: binding.classNamesImportName,
        bindingDeclId: resolveBindingDeclId(binding, sourceBindingGraph, sourceFile, index),
      },
    ];
  });
}

function resolveBindingDeclId(
  binding: CxBinding,
  sourceBindingGraph: SourceBindingGraph | undefined,
  sourceFile: ts.SourceFile | undefined,
  index: number,
): string {
  if (!sourceBindingGraph || !sourceFile) {
    return `synthetic-binding-decl:${index}`;
  }

  const resolution = resolveBindingAtOffset(
    sourceBindingGraph,
    binding.cxVarName,
    ts.getPositionOfLineAndCharacter(
      sourceFile,
      binding.bindingRange.start.line,
      binding.bindingRange.start.character,
    ),
  );
  return resolution?.declId ?? `synthetic-binding-decl:${index}`;
}

function isValidImportedBinding(
  binding: CxBinding,
  sourceBindingGraph: SourceBindingGraph | undefined,
): boolean {
  if (
    !sourceBindingGraph ||
    binding.classNamesReferenceOffset === undefined ||
    binding.stylesReferenceOffset === undefined
  ) {
    return true;
  }

  const expectedClassNamesDeclId = findImportBindingDeclId(
    sourceBindingGraph,
    binding.classNamesImportName,
    new Set(["classnames/bind"]),
  );
  const expectedStylesDeclId = findImportBindingDeclId(sourceBindingGraph, binding.stylesVarName);
  if (!expectedClassNamesDeclId || !expectedStylesDeclId) {
    return false;
  }

  const classNamesResolution = resolveBindingAtOffset(
    sourceBindingGraph,
    binding.classNamesImportName,
    binding.classNamesReferenceOffset,
  );
  const stylesResolution = resolveBindingAtOffset(
    sourceBindingGraph,
    binding.stylesVarName,
    binding.stylesReferenceOffset,
  );

  return (
    classNamesResolution?.declId === expectedClassNamesDeclId &&
    stylesResolution?.declId === expectedStylesDeclId
  );
}
