import type { CmeCheckerBundleV0 } from "./shadow";

export interface CmeCheckerTestkitArchetypeV0 {
  readonly label: string;
  readonly bundle: CmeCheckerBundleV0;
  readonly category: "source" | "style";
  readonly expectedCode: string;
  readonly fixture: string;
}

export const OMENA_CHECKER_TESTKIT_ARCHETYPES = [
  {
    label: "testkit-source-missing-static-class",
    bundle: "source-missing",
    category: "source",
    expectedCode: "missing-static-class",
    fixture: `//- src/App.jsx layer:source
import classNames from "classnames/bind";
import styles from "./App.module.scss";

const cx = classNames.bind(styles);

export function App() {
  return <div className={cx("chip", "ghost")}>hi</div>;
}
//- src/App.module.scss dialect:scss layer:style consumer-of:src/App.jsx
.chip { color: red; }
--- expect: product
cme-checker.source-missing
--- expect: code
missing-static-class
`,
  },
  {
    label: "testkit-style-unused-selector",
    bundle: "style-unused",
    category: "style",
    expectedCode: "unused-selector",
    fixture: `//- src/App.tsx layer:source
import styles from "./App.module.css";

export function App() {
  return <div className={styles.used} />;
}
//- src/App.module.css dialect:css layer:style consumer-of:src/App.tsx
.used { color: red; }
.unused { color: blue; }
--- expect: product
cme-checker.style-unused
--- expect: code
unused-selector
`,
  },
  {
    label: "testkit-style-recovery-missing-composed-module",
    bundle: "style-recovery",
    category: "style",
    expectedCode: "missing-composed-module",
    fixture: `//- src/App.module.css dialect:css layer:style
.button {
  composes: base from "./Missing.module.css";
  color: red;
}
--- expect: product
cme-checker.style-recovery
--- expect: code
missing-composed-module
`,
  },
] as const satisfies readonly CmeCheckerTestkitArchetypeV0[];
