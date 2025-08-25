use wallet_adapter_common::{
    feature_support::FeatureSupport,
    standardized_events::{
        SOLANA_SIGN_AND_SEND_TRANSACTION_IDENTIFIER, SOLANA_SIGN_IN_IDENTIFIER,
        SOLANA_SIGN_MESSAGE_IDENTIFIER, SOLANA_SIGN_TRANSACTION_IDENTIFIER,
        STANDARD_CONNECT_IDENTIFIER, STANDARD_DISCONNECT_IDENTIFIER, STANDARD_EVENTS_IDENTIFIER,
    },
};

use crate::{
    Connect, Disconnect, Reflection, SemverVersion, SignIn, SignMessage, SignTransaction,
    StandardEvents, WalletError, WalletResult,
};

/// All the features of `standard` and `solana` events as specified
/// in the wallet standard. These features are contained within a
/// [crate::Wallet] struct allowing to check if a wallet supports a certain
/// feature and also calling the callback functions to make requests to a browser wallet.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Features {
    /// standard:connect
    pub(crate) connect: Connect,
    /// standard:disconnect
    pub(crate) disconnect: Disconnect,
    /// standard:events
    pub(crate) events: StandardEvents,
    /// solana:signAndSendTransaction
    pub(crate) sign_and_send_tx: SignTransaction,
    /// solana:signTransaction
    pub(crate) sign_tx: SignTransaction,
    /// solana:signMessage
    pub(crate) sign_message: SignMessage,
    /// solana:signIn
    pub(crate) sign_in: Option<SignIn>,
    /// Non-standard features
    extensions: Vec<String>,
}

impl Features {
    /// Parse all the features from a wallet described as a [web_sys::wasm_bindgen::JsValue]
    pub(crate) fn parse(reflection: &Reflection) -> WalletResult<(Self, FeatureSupport)> {
        let features_keys = reflection.object_to_vec_string("features")?;
        let features_object = Reflection::new_from_str(reflection.get_inner(), "features")?;

        let mut features = Features::default();
        let mut supported_features = FeatureSupport::default();

        features_keys.into_iter().try_for_each(|feature| {
            let inner_object = features_object.reflect_inner(&feature)?;
            let inner_object = Reflection::new(inner_object)?;

            if feature.starts_with("standard:") || feature.starts_with("solana:") {
                let version = SemverVersion::from_jsvalue(&inner_object)?;

                if feature == STANDARD_CONNECT_IDENTIFIER {
                    features.connect = Connect::new(&inner_object, version)?;
                    supported_features.connect = true;
                } else if feature == STANDARD_DISCONNECT_IDENTIFIER {
                    features.disconnect = Disconnect::new(&inner_object, version)?;
                    supported_features.disconnect = true;
                } else if feature == STANDARD_EVENTS_IDENTIFIER {
                    features.events = StandardEvents::new(&inner_object, version)?;
                    supported_features.events = true;
                } else if feature == SOLANA_SIGN_AND_SEND_TRANSACTION_IDENTIFIER {
                    features.sign_and_send_tx =
                        SignTransaction::new_sign_and_send_tx(&inner_object, version)?;
                    supported_features.sign_and_send_tx = true;
                } else if feature == SOLANA_SIGN_TRANSACTION_IDENTIFIER {
                    features.sign_tx = SignTransaction::new_sign_tx(&inner_object, version)?;
                    supported_features.sign_tx = true;
                } else if feature == SOLANA_SIGN_MESSAGE_IDENTIFIER {
                    features.sign_message = SignMessage::new(&inner_object, version)?;
                    supported_features.sign_message = true;
                } else if feature == SOLANA_SIGN_IN_IDENTIFIER {
                    features
                        .sign_in
                        .replace(SignIn::new(&inner_object, version)?);
                    supported_features.sign_in = true;
                } else {
                    return Err(WalletError::UnsupportedWalletFeature(feature));
                }
            } else {
                features.extensions.push(feature);
            }

            Ok(())
        })?;

        Ok((features, supported_features))
    }

    /// Get all extensions on the wallet
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }
}
