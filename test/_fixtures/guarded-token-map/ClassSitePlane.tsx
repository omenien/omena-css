import clsx from "clsx";
import styles from "./ClassSitePlane.module.scss";

declare function computeClassName(): string;

export function ClassSitePlane({ flag }: { flag: boolean }) {
  const size = flag ? "large" : "small";
  return (
    <section>
      <div className="root root" />
      <div class="legacy legacy" />
      <div className={flag ? "active" : "idle"} />
      <div className={`btn-${size}`} />
      <div className={clsx("card", "raised")} />
      <div className={clsx({ active: flag })} />
      <div className={computeClassName()} />
      <div className={styles.root} />
    </section>
  );
}
