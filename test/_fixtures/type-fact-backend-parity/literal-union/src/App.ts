import bind from "classnames/bind";
import styles from "./App.module.scss";

const cx = bind.bind(styles);

type Variant = "button-primary" | "button-secondary";

const variant: Variant = Math.random() > 0.5 ? "button-primary" : "button-secondary";

declare function pickTone(): "call-primary" | "call-secondary";
const themeByKey = { alpha: "computed-alpha", beta: "computed-beta" } as const;
declare const themeKey: "alpha" | "beta";
declare const nestedVariant: "small-soft" | "large-strong";
declare const enabled: boolean;
declare function logicalTone(): "logical-only";

export const callTypeFact = pickTone();
export const computedTypeFact = themeByKey[themeKey];
export const nestedTypeFact = `${`nested-${nestedVariant}` as const}`;
export const arithmeticTypeFact = "arithmetic-" + variant;
export const logicalTypeFact = `${enabled && logicalTone()}`;
export const className = cx(variant);
