import styles from "$styles/Card.module.scss";

export function App() {
  return (
    <main className={styles.card}>
      <h1 className={styles.title}>Workspace lint fixture</h1>
      <div className={styles.utility}>Utility row</div>
    </main>
  );
}
