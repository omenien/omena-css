# omena-transform-egg

`omena-transform-egg` owns the optional e-graph rewrite boundary for P08 and
P25. The core crate does not pull in an e-graph engine yet; this boundary
records the proof obligations that future egg-backed rewrites must satisfy
before they can enter the transform DAG.
