use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Reflect;

use crate::{
    Reflection, SemverVersion, WalletAccount, WalletError, WalletResult, STANDARD_CONNECT,
};

use super::Connect;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FeatureInfo {
    pub version: SemverVersion,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FeatureInfoWithTx {
    pub version: SemverVersion,
    pub legacy: bool,
    pub version_zero: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Features {
    /// standard:connect
    connect: Connect,
    /// standard:disconnect
    disconnect: Option<FeatureInfo>,
    /// standard:events
    events: Option<FeatureInfo>,
    /// solana:signAndSendTransaction
    sign_and_send_tx: Option<FeatureInfoWithTx>,
    /// solana:signTransaction
    sign_tx: Option<FeatureInfoWithTx>,
    /// solana:signMessage
    sign_message: Option<FeatureInfo>,
    /// solana:signIn
    sign_in: Option<FeatureInfo>,
    /// Non-standard features
    extensions: Vec<String>,
}

impl Features {
    pub fn parse(reflection: &Reflection) -> WalletResult<Self> {
        let features_keys = reflection.object_to_vec_string("features")?;
        let features_object = Reflection::new_from_str(reflection.get_inner(), "features")?;

        let mut features = Features::default();

        features_keys.into_iter().try_for_each(|feature| {
            let inner_object = features_object.reflect_inner(&feature)?;

            if feature.starts_with("standard:") || feature.starts_with("solana:") {
                let version = SemverVersion::from_jsvalue(&inner_object)?;

                let get_tx_version_support =
                    |value: &JsValue, version: SemverVersion| -> WalletResult<FeatureInfoWithTx> {
                        let tx_version_support_jsvalue =
                            Reflect::get(value, &"supportedTransactionVersions".into()).or(Err(
                                WalletError::ExpectedValueNotFound(
                                    "supportedTransactionVersions".to_string(),
                                ),
                            ))?;
                        let tx_version_support = tx_version_support_jsvalue
                            .dyn_ref::<js_sys::Array>()
                            .ok_or(WalletError::ExpectedArray(
                                "supportedTransactionVersions".to_string(),
                            ))?;

                        let mut tx_info = FeatureInfoWithTx {
                            version,
                            legacy: false,
                            version_zero: false,
                        };

                        tx_version_support.iter().try_for_each(|value| {
                            if value == JsValue::from_str("legacy") {
                                tx_info.legacy = true;
                            } else if value == JsValue::from(0) {
                                tx_info.version_zero = true;
                            } else {
                                return Err(WalletError::UnsupportedTransactionVersion);
                            }

                            Ok(())
                        })?;

                        if tx_info.legacy != true {
                            return Err(WalletError::LegacyTransactionSupportRequired);
                        }

                        Ok(tx_info)
                    };

                if feature == STANDARD_CONNECT {
                    features.connect = Connect::new(inner_object, version)?;
                } else if feature == "standard:disconnect" {
                    features.disconnect.replace(FeatureInfo { version });
                } else if feature == "standard:events" {
                    features.events.replace(FeatureInfo { version });
                } else if feature == "solana:signAndSendTransaction" {
                    features
                        .sign_and_send_tx
                        .replace(get_tx_version_support(&inner_object, version)?);
                } else if feature == "solana:signTransaction" {
                    features
                        .sign_tx
                        .replace(get_tx_version_support(&inner_object, version)?);
                } else if feature == "solana:signMessage" {
                    features.sign_message.replace(FeatureInfo { version });
                } else if feature == "solana:signIn" {
                    features.sign_in.replace(FeatureInfo { version });
                } else {
                    return Err(WalletError::UnsupportedStandardFeature(feature));
                }
            } else {
                features.extensions.push(feature);
            }

            Ok(())
        })?;

        Ok(features)
    }

    pub async fn connect(&self) -> WalletResult<Vec<WalletAccount>> {
        self.connect.call_connect().await
    }

    pub fn disconnect(&self) -> Option<&FeatureInfo> {
        self.disconnect.as_ref()
    }

    pub fn events(&self) -> Option<&FeatureInfo> {
        self.events.as_ref()
    }

    pub fn sign_and_send_transaction(&self) -> Option<&FeatureInfoWithTx> {
        self.sign_and_send_tx.as_ref()
    }

    pub fn sign_transaction(&self) -> Option<&FeatureInfoWithTx> {
        self.sign_tx.as_ref()
    }

    pub fn sign_message(&self) -> Option<&FeatureInfo> {
        self.sign_message.as_ref()
    }

    pub fn sign_in(&self) -> Option<&FeatureInfo> {
        self.sign_in.as_ref()
    }

    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }
}
