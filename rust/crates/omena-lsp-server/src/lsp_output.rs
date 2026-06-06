use serde_json::Value;

pub const OPTIMIZING_DIAGNOSTICS_DELAY_MS: u64 = 200;

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLspOutput {
    pub value: Value,
    pub delay_millis: Option<u64>,
    pub coalesce_key: Option<String>,
}

impl ScheduledLspOutput {
    pub fn immediate(value: Value) -> Self {
        Self {
            value,
            delay_millis: None,
            coalesce_key: None,
        }
    }

    pub fn immediate_coalesced(value: Value, coalesce_key: String) -> Self {
        Self {
            value,
            delay_millis: None,
            coalesce_key: Some(coalesce_key),
        }
    }

    pub fn delayed(value: Value, delay_millis: u64) -> Self {
        Self {
            value,
            delay_millis: Some(delay_millis),
            coalesce_key: None,
        }
    }

    pub fn delayed_coalesced(value: Value, delay_millis: u64, coalesce_key: String) -> Self {
        Self {
            value,
            delay_millis: Some(delay_millis),
            coalesce_key: Some(coalesce_key),
        }
    }

    pub fn into_value(self) -> Value {
        self.value
    }
}
