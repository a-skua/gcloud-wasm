pub(crate) enum Adc {
    AuthorizedUser {
        client_id: String,
        client_secret: String,
        refresh_token: String,
    },
    ServiceAccount {
        client_email: String,
        private_key: String,
        private_key_id: String,
        token_uri: String,
    },
}

impl<'de> serde::Deserialize<'de> for Adc {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            r#type: String,
            // authorized_user fields
            client_id: Option<String>,
            client_secret: Option<String>,
            refresh_token: Option<String>,
            // service_account fields
            client_email: Option<String>,
            private_key: Option<String>,
            private_key_id: Option<String>,
            token_uri: Option<String>,
        }

        let raw = Raw::deserialize(deserializer)?;
        match raw.r#type.as_str() {
            "authorized_user" => Ok(Adc::AuthorizedUser {
                client_id: raw
                    .client_id
                    .ok_or_else(|| serde::de::Error::missing_field("client_id"))?,
                client_secret: raw
                    .client_secret
                    .ok_or_else(|| serde::de::Error::missing_field("client_secret"))?,
                refresh_token: raw
                    .refresh_token
                    .ok_or_else(|| serde::de::Error::missing_field("refresh_token"))?,
            }),
            "service_account" => Ok(Adc::ServiceAccount {
                client_email: raw
                    .client_email
                    .ok_or_else(|| serde::de::Error::missing_field("client_email"))?,
                private_key: raw
                    .private_key
                    .ok_or_else(|| serde::de::Error::missing_field("private_key"))?,
                private_key_id: raw
                    .private_key_id
                    .ok_or_else(|| serde::de::Error::missing_field("private_key_id"))?,
                token_uri: raw
                    .token_uri
                    .unwrap_or_else(|| "https://oauth2.googleapis.com/token".to_string()),
            }),
            other => Err(serde::de::Error::custom(format!(
                "unsupported credential type: {other}"
            ))),
        }
    }
}

pub(crate) fn read_adc() -> Result<Adc, String> {
    let adc_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").unwrap_or_else(|_| {
        let home = std::env::var("HOME").expect("HOME is not set");
        format!("{home}/.config/gcloud/application_default_credentials.json")
    });

    let json = std::fs::read_to_string(&adc_path)
        .map_err(|e| format!("failed to read ADC file {adc_path}: {e}"))?;

    serde_json::from_str(&json).map_err(|e| format!("failed to parse ADC file: {e}"))
}
