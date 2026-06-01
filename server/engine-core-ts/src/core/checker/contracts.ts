import type { Range } from "@omena/shared";
import type { FlowResolution } from "../flow/lattice";
import type { ReducedClassValueDerivation } from "../query/read-expression-semantics";

export type CheckerSeverity = "warning" | "hint";

export type CheckerFinding = SourceCheckerFinding | StyleCheckerFinding;

export interface WorkspaceCheckerFinding {
  readonly filePath: string;
  readonly finding: CheckerFinding;
}

export type SourceCheckerFinding =
  | {
      readonly category: "source";
      readonly code: "missing-module";
      readonly severity: "warning";
      readonly range: Range;
      readonly specifier: string;
      readonly absolutePath: string;
    }
  | {
      readonly category: "source";
      readonly code: "missing-static-class";
      readonly severity: "warning";
      readonly range: Range;
      readonly scssModulePath: string;
      readonly className: string;
      readonly suggestion?: string;
    }
  | {
      readonly category: "source";
      readonly code: "missing-template-prefix";
      readonly severity: "warning";
      readonly range: Range;
      readonly scssModulePath: string;
      readonly staticPrefix: string;
    }
  | {
      readonly category: "source";
      readonly code: "missing-resolved-class-values";
      readonly severity: "warning";
      readonly range: Range;
      readonly scssModulePath: string;
      readonly missingValues: readonly string[];
      readonly abstractValue: FlowResolution["abstractValue"];
      readonly valueCertainty: "exact" | "inferred" | "possible";
      readonly selectorCertainty: "exact" | "inferred" | "possible";
      readonly reason: "flowLiteral" | "flowBranch" | "typeUnion";
      readonly valueDomainDerivation?: ReducedClassValueDerivation;
    }
  | {
      readonly category: "source";
      readonly code: "missing-resolved-class-domain";
      readonly severity: "warning";
      readonly range: Range;
      readonly scssModulePath: string;
      readonly abstractValue: FlowResolution["abstractValue"];
      readonly valueCertainty: "exact" | "inferred" | "possible";
      readonly selectorCertainty: "exact" | "inferred" | "possible";
      readonly reason: "flowLiteral" | "flowBranch" | "typeUnion";
      readonly valueDomainDerivation?: ReducedClassValueDerivation;
    };

export type StyleCheckerFinding =
  | {
      readonly category: "style";
      readonly code: "unused-selector";
      readonly severity: "hint";
      readonly range: Range;
      readonly selectorFilePath: string;
      readonly canonicalName: string;
    }
  | {
      readonly category: "style";
      readonly code: "missing-composed-module";
      readonly severity: "warning";
      readonly range: Range;
      readonly selectorFilePath: string;
      readonly fromSpecifier?: string;
      readonly targetFilePath: string;
    }
  | {
      readonly category: "style";
      readonly code: "missing-composed-selector";
      readonly severity: "warning";
      readonly range: Range;
      readonly selectorFilePath: string;
      readonly fromSpecifier?: string;
      readonly targetFilePath: string;
      readonly className: string;
    }
  | {
      readonly category: "style";
      readonly code: "missing-value-module";
      readonly severity: "warning";
      readonly range: Range;
      readonly selectorFilePath: string;
      readonly fromSpecifier: string;
      readonly targetFilePath: string;
    }
  | {
      readonly category: "style";
      readonly code: "missing-imported-value";
      readonly severity: "warning";
      readonly range: Range;
      readonly selectorFilePath: string;
      readonly fromSpecifier: string;
      readonly targetFilePath: string;
      readonly importedName: string;
      readonly localName: string;
    }
  | {
      readonly category: "style";
      readonly code: "missing-keyframes";
      readonly severity: "warning";
      readonly range: Range;
      readonly selectorFilePath: string;
      readonly animationName: string;
    }
  | {
      readonly category: "style";
      readonly code: "missing-custom-property";
      readonly severity: "warning";
      readonly range: Range;
      readonly selectorFilePath: string;
      readonly propertyName: string;
    }
  | {
      readonly category: "style";
      readonly code: "missing-sass-symbol";
      readonly severity: "warning";
      readonly range: Range;
      readonly selectorFilePath: string;
      readonly selectorName: string;
      readonly symbolSyntax?: "sass" | "less";
      readonly symbolKind: "variable" | "mixin" | "function";
      readonly symbolName: string;
      readonly symbolRole: "reference" | "include" | "call";
    };
