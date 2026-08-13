declare module "classnames/bind" {
  type Binder = (...values: readonly unknown[]) => string;
  const classNamesBind: {
    bind(styles: Record<string, string>): Binder;
  };
  export default classNamesBind;
}
