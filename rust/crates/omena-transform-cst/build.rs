use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

const STABLE_NODE_KEY_STRING_ARM_EXPIRY_UNIX_DAY: u64 = 20_727;

fn main() {
    println!("cargo::rerun-if-env-changed=SOURCE_DATE_EPOCH");
    println!("cargo::rustc-check-cfg=cfg(stable_node_key_string_arm_expired)");

    let now_seconds = env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0)
        });
    if now_seconds / 86_400 >= STABLE_NODE_KEY_STRING_ARM_EXPIRY_UNIX_DAY {
        println!("cargo::rustc-cfg=stable_node_key_string_arm_expired");
    }
}
