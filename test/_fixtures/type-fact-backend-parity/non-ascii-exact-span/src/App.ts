import bind from "classnames/bind";
import styles from "./App.module.scss";

const cx = bind.bind(styles);

/** 카드 흐름 → the exact-span target deliberately follows non-ASCII source text. */
type Tone = "card-large" | "card-small";
const tone: Tone = Math.random() > 0.5 ? "card-large" : "card-small";

export const className = cx(tone);
