import classNames from "classnames/bind";
import styles from "./SourcePrefixSuffixOverlapParity.module.scss";

const cx = classNames.bind(styles);

function resolveClassName(value: number) {
  switch (value) {
    case 1:
      return "ab-cd";
    case 2:
      return "ab-x-cd";
    case 3:
      return "ab-long-cd";
    case 4:
      return "ab-wide-cd";
    case 5:
      return "ab-ping-cd";
    case 6:
      return "ab-zone-cd";
    case 7:
      return "ab-card-cd";
    case 8:
      return "ab-tone-cd";
    default:
      return "ab-ruby-cd";
  }
}

export function SourcePrefixSuffixOverlapParity(value: number) {
  const className = resolveClassName(value);
  return <div className={cx(className)} />;
}
