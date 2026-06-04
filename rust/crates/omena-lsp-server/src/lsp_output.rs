use serde_json::Value;

pub const OPTIMIZING_DIAGNOSTICS_DELAY_MS: u64 = 200;

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLspOutput {
    pub value: Value,
    pub delay_millis: Option<u64>,
}

impl ScheduledLspOutput {
    pub fn immediate(value: Value) -> Self {
        Self {
            value,
            delay_millis: None,
        }
    }

    pub fn delayed(value: Value, delay_millis: u64) -> Self {
        Self {
            value,
            delay_millis: Some(delay_millis),
        }
    }

    pub fn into_value(self) -> Value {
        self.value
    }
}
