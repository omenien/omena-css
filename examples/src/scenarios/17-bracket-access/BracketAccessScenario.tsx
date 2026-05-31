import styles from "./BracketAccess.module.scss";

/**
 * 17 · bracket access
 *
 * Manual QA targets:
 * - `styles["btn-primary"]`
 * - `styles["accent-pill"]`
 * - `styles["한글-라벨"]`
 * - optional `classnameTransform` alias checks on the dashed names
 */
export function BracketAccessScenario() {
  return (
    <div style={{ display: "grid", gap: 16, maxWidth: 620 }}>
      <button type="button" className={styles["btn-primary"]}>
        Bracket access on dashed selectors
      </button>

      <div className={styles["accent-pill"]}>
        <span>accent-pill</span>
        <code>styles["accent-pill"]</code>
      </div>

      <div className={styles["한글-라벨"]}>
        <span>한글-라벨</span>
        <code>styles["한글-라벨"]</code>
      </div>

      <p className={styles.meta}>
        If you switch <code>omena.scss.classnameTransform</code> to{" "}
        <code>camelCase</code>, you can also verify alias resolution on the dashed selectors
        without changing this scenario's runtime behavior.
      </p>
    </div>
  );
}
