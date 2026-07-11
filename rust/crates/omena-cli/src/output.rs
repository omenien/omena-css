use omena_query::{
    OmenaCliResponseEnvelopeV0, OmenaError, OmenaErrorClassV0, OmenaErrorContextV0,
    OmenaErrorRecoverabilityV0, OmenaErrorSeverityV0, OmenaSdkErrorEnvelopeV0,
};
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
    metadata: CliOutputMetadataV0<'_>,
    value: &T,
) -> Result<(), String> {
    let json = serialize_json_envelope(metadata, value)?;
    println!("{json}");
    Ok(())
}

fn serialize_json_envelope<T: Serialize>(
    metadata: CliOutputMetadataV0<'_>,
    value: &T,
) -> Result<String, String> {
    let payload = serde_json::to_value(value)
        .map_err(|error| serialize_json_error_envelope(metadata.product, error))?;
    serde_json::to_string_pretty(&OmenaCliResponseEnvelopeV0 {
        schema_version: "0".to_string(),
        product: metadata.product.to_string(),
        config_content_digest: metadata.config_content_digest.map(str::to_string),
        payload,
    })
    .map_err(|error| serialize_json_error_envelope(metadata.product, error))
}

fn serialize_json_error_envelope(product: &str, error: serde_json::Error) -> String {
    let envelope = OmenaSdkErrorEnvelopeV0 {
        error: OmenaError::new(
            OmenaErrorClassV0::Internal,
            format!("failed to serialize {product} JSON response: {error}"),
            OmenaErrorContextV0 {
                code: "cli.output.serialize".to_string(),
                severity: OmenaErrorSeverityV0::Error,
                recoverability: OmenaErrorRecoverabilityV0::Retry,
            },
        ),
    };
    serde_json::to_string(&envelope).unwrap_or_else(|_| {
        "{\"error\":{\"class\":\"internal\",\"message\":\"failed to serialize CLI JSON response\",\"context\":{\"code\":\"cli.output.serialize\",\"severity\":\"error\",\"recoverability\":\"retry\"}}}".to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Serializer, ser::Error as _};

    #[derive(Serialize)]
    struct Fixture {
        value: u8,
    }

    #[test]
    fn response_envelope_carries_product_payload_and_config_digest() -> Result<(), String> {
        let payload = Fixture { value: 7 };
        let metadata = CliOutputMetadataV0::new("omena-cli.fixture")
            .with_config_content_digest(Some("config-digest"));
        let json = serialize_json_envelope(metadata, &payload)?;
        let envelope: OmenaCliResponseEnvelopeV0 =
            serde_json::from_str(&json).map_err(|error| error.to_string())?;

        assert_eq!(envelope.schema_version, "0");
        assert_eq!(envelope.product, "omena-cli.fixture");
        assert_eq!(
            envelope.config_content_digest.as_deref(),
            Some("config-digest")
        );
        assert_eq!(envelope.payload, serde_json::json!({ "value": 7 }));
        Ok(())
    }

    struct UnserializableFixture;

    impl Serialize for UnserializableFixture {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(S::Error::custom("fixture serialization failure"))
        }
    }

    #[test]
    fn serialization_failure_uses_the_typed_error_envelope() -> Result<(), String> {
        let Err(error) = serialize_json_envelope(
            CliOutputMetadataV0::new("omena-cli.fixture"),
            &UnserializableFixture,
        ) else {
            return Err("fixture must fail serialization".to_string());
        };
        let envelope: OmenaSdkErrorEnvelopeV0 =
            serde_json::from_str(&error).map_err(|parse_error| parse_error.to_string())?;

        assert_eq!(envelope.error.class, OmenaErrorClassV0::Internal);
        assert_eq!(envelope.error.context.code, "cli.output.serialize");
        assert!(envelope.error.message.contains("omena-cli.fixture"));
        Ok(())
    }
}
