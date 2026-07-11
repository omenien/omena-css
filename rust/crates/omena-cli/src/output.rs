use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CliOutputMetadataV0<'a> {
    pub(crate) product: &'static str,
    pub(crate) config_content_digest: Option<&'a str>,
}

impl<'a> CliOutputMetadataV0<'a> {
    pub(crate) const fn new(product: &'static str) -> Self {
        Self {
            product,
            config_content_digest: None,
        }
    }

    pub(crate) const fn with_config_content_digest(
        self,
        config_content_digest: Option<&'a str>,
    ) -> Self {
        Self {
            config_content_digest,
            ..self
        }
    }
}

pub(crate) fn print_json<T: Serialize>(
    _metadata: CliOutputMetadataV0<'_>,
    value: &T,
) -> Result<(), String> {
    let json = serialize_json_payload(value)?;
    println!("{json}");
    Ok(())
}

fn serialize_json_payload<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to serialize JSON: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Fixture {
        value: u8,
    }

    #[test]
    fn metadata_does_not_change_payload_bytes_before_envelope_routing() -> Result<(), String> {
        let payload = Fixture { value: 7 };
        let plain = serialize_json_payload(&payload)?;
        let metadata = CliOutputMetadataV0::new("omena-cli.fixture")
            .with_config_content_digest(Some("config-digest"));

        assert_eq!(metadata.product, "omena-cli.fixture");
        assert_eq!(metadata.config_content_digest, Some("config-digest"));
        assert_eq!(plain, "{\n  \"value\": 7\n}");
        Ok(())
    }
}
