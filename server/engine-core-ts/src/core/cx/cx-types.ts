import type { Range } from "@omena/shared";

/**
 * One raw `cx = classNames.bind(styles)` binding discovered during
 * the source import/binding scan.
 *
 * This is an analysis-internal fact. Downstream layers should prefer
 * binder-linked identities (`ResolvedCxBinding`, HIR utility bindings)
 * over this raw scan output.
 */
export interface CxBinding {
  /** Identifier used at call sites — `cx`, `classes`, `cxBtn`, etc. */
  readonly cxVarName: string;
  /** Identifier for the styles default-import. */
  readonly stylesVarName: string;
  /** Absolute path of the `.module.scss|css` file the binding resolves to. */
  readonly scssModulePath: string;
  /** Source range covering the binding identifier declaration. */
  readonly bindingRange: Range;
  /** Source offset of the binding identifier declaration. */
  readonly bindingDeclOffset?: number;
  /**
   * Identifier the `classnames/bind` default import was bound to in
   * this file. Usually `classNames`, but aliased imports allow any
   * name.
   */
  readonly classNamesImportName: string;
  /** Source offset of the `classnames/bind` identifier in `<cn>.bind(...)`. */
  readonly classNamesReferenceOffset?: number;
  /** Source offset of the styles identifier in `.bind(<styles>)`. */
  readonly stylesReferenceOffset?: number;
}
