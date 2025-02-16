use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthOptions {
    /// Enables authentication using the device's password. This feature is available on both Android and iOS.
    pub allow_device_credential: bool,
    /// Label for the Cancel button. This feature is available on both Android and iOS.
    pub cancel_title: Option<String>,
    /// The plain data that must be encrypted after successfull biometric authentication 
    pub data_to_encrypt: Option<String>,
    /// The encrypted data that must be decrypted after successfull biometric authentication 
    pub data_to_decrypt: Option<String>,
    /// Specifies the text displayed on the fallback button if biometric authentication fails. This feature is available iOS only.
    pub fallback_title: Option<String>,
    /// Title indicating the purpose of biometric verification. This feature is available Android only.
    pub title: Option<String>,
    /// SubTitle providing contextual information of biometric verification. This feature is available Android only.
    pub subtitle: Option<String>,
    /// Specifies whether additional user confirmation is required, such as pressing a button after successful biometric authentication. This feature is available Android only.
    pub confirmation_required: Option<bool>,
}

#[cfg(mobile)]
impl TryInto<tauri_plugin_biometric::AuthOptions> for AuthOptions {
    type Error = &'static str;

    fn try_into(self) -> Result<tauri_plugin_biometric::AuthOptions, Self::Error> {
        Ok(tauri_plugin_biometric::AuthOptions {
            allow_device_credential: self.allow_device_credential,
            cancel_title: self.cancel_title,
            data_to_encrypt: self.data_to_encrypt,
            data_to_decrypt: self.data_to_decrypt,
            fallback_title: self.fallback_title,
            title: self.title,
            subtitle: self.subtitle,
            confirmation_required: self.confirmation_required,
        })
    }
}
