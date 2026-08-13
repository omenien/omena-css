import classNames from "classnames/bind";
import styles from "./SourceUnicodeLengthParity.module.scss";

const cx = classNames.bind(styles);

function resolveClassName(value: number) {
  switch (value) {
    case 1:
      return "카드-long-활성";
    case 2:
      return "카드-wide-활성";
    case 3:
      return "카드-ping-활성";
    case 4:
      return "카드-zone-활성";
    case 5:
      return "카드-xxxx-활성";
    case 6:
      return "카드-card-활성";
    case 7:
      return "카드-tone-활성";
    case 8:
      return "카드-mild-활성";
    default:
      return "카드-ruby-활성";
  }
}

export function SourceUnicodeLengthParity(value: number) {
  const className = resolveClassName(value);
  return <div className={cx(className)} />;
}
