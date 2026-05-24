pub fn nats_to_bits(value_in_nats: f64) -> f64 {
    value_in_nats / std::f64::consts::LN_2
}

pub fn bits_to_nats(value_in_bits: f64) -> f64 {
    value_in_bits * std::f64::consts::LN_2
}
