import styles from "./Missing.module.scss";

export function BrokenImport() {
  return <aside className={styles.missing}>Missing module fixture</aside>;
}
