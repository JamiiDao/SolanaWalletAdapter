use std::borrow::Cow;

use wallet_adapter_common::{
    chains::ChainSupport,
    clusters::Cluster,
    feature_support::FeatureSupport,
    standardized_events::{
        SOLANA_SIGN_AND_SEND_TRANSACTION_IDENTIFIER, SOLANA_SIGN_IN_IDENTIFIER,
        SOLANA_SIGN_MESSAGE_IDENTIFIER, SOLANA_SIGN_TRANSACTION_IDENTIFIER,
        STANDARD_CONNECT_IDENTIFIER, STANDARD_DISCONNECT_IDENTIFIER, STANDARD_EVENTS_IDENTIFIER,
    },
    WalletAccountData, WalletCommonUtils,
};
use web_sys::wasm_bindgen::JsValue;

use crate::{Reflection, WalletError, WalletIcon, WalletResult};

/// Interface of a **WalletAccount**, also referred to as an **Account**.
/// An account is a _read-only data object_ that is provided from the Wallet to the app,
/// authorizing the app to use it.
/// The app can use an account to display and query information from a chain.
/// The app can also act using an account by passing it to `features` field of the Wallet.
#[derive(Clone, Default, PartialEq)]
pub struct WalletAccount {
    pub(crate) account: WalletAccountData,
    /// The Javascript Value Representation of a wallet,
    /// this mostly used internally in the wallet adapter
    pub(crate) js_value: JsValue,
}

impl WalletAccount {
    /// Address of the account, corresponding with a public key.
    pub fn address(&self) -> &str {
        self.account.address.as_str()
    }

    /// Public key of the account, corresponding with a secret key to use.
    pub fn public_key(&self) -> [u8; 32] {
        self.account.public_key
    }

    /// Chains supported by the account.
    /// This must be a subset of the {@link Wallet.chains | chains} of the Wallet.
    pub fn chains(&self) -> &[String] {
        self.account.chains.as_slice()
    }

    /// Feature names supported by the account.
    /// This must be a subset of the names of {@link Wallet.features | features} of the Wallet.
    pub fn features(&self) -> &[String] {
        self.account.features.as_slice()
    }

    /// Optional user-friendly descriptive label or name for the account. This may be displayed by the app.
    pub fn label(&self) -> Option<&String> {
        self.account.label.as_ref()
    }

    /// An optional [Wallet Icon](String)
    pub fn icon(&self) -> Option<&String> {
        self.account.icon.as_ref()
    }

    /// Get the shortened address of the `Base58 address` .
    /// It displays the first 4 characters and the last for characters
    /// separated by ellipsis eg `FXdl...RGd4` .
    /// If the address is less than 8 characters, an error is thrown
    pub fn shorten_address<'a>(&'a self) -> WalletResult<Cow<'a, str>> {
        Ok(WalletCommonUtils::shorten_base58(&self.account.address)?)
    }

    /// Same as [Self::shorten_address] but with a custom range
    /// instead of taking the first 4 character and the last 4 characters
    /// it uses a custom range.
    pub fn custom_shorten_address<'a>(&'a self, take: usize) -> WalletResult<Cow<'a, str>> {
        Ok(WalletCommonUtils::custom_shorten_base58(
            &self.account.address,
            take,
        )?)
    }

    /// Same as [Self::shorten_address] but with a custom range
    /// instead of taking the first 4 character and the last 4 characters
    /// it uses a custom range for first characters before ellipsis and last characters after ellipsis.
    pub fn custom_shorten_address_rl<'a>(
        &'a self,
        left: usize,
        right: usize,
    ) -> WalletResult<Cow<'a, str>> {
        Ok(WalletCommonUtils::custom_shorten_address_rl(
            self.account.address(),
            left,
            right,
        )?)
    }

    /// Parse A [WalletAccount] from [JsValue]
    pub(crate) fn parse(reflection: Reflection) -> WalletResult<Self> {
        let address = reflection.string("address")?;
        let public_key = reflection.byte32array("publicKey")?;
        let chains = reflection.vec_string_accept_undefined("chains")?;
        let features = reflection.vec_string_accept_undefined("features")?;

        let mut supported_chains = ChainSupport::default();

        chains.iter().try_for_each(|chain| {
            if chain.as_str() == Cluster::MainNet.chain() {
                supported_chains.mainnet = true;
            } else if chain.as_str() == Cluster::DevNet.chain() {
                supported_chains.devnet = true;
            } else if chain.as_str() == Cluster::TestNet.chain() {
                supported_chains.testnet = true;
            } else if chain.as_str() == Cluster::LocalNet.chain() {
                supported_chains.localnet = true;
            } else {
                return Err(WalletError::UnsupportedChain(chain.to_owned()));
            }

            Ok(())
        })?;

        let mut supported_features = FeatureSupport::default();

        features.iter().try_for_each(|feature| {
            if feature.as_str() == STANDARD_CONNECT_IDENTIFIER {
                supported_features.connect = true;
            } else if feature.as_str() == STANDARD_DISCONNECT_IDENTIFIER {
                supported_features.disconnect = true;
            } else if feature.as_str() == STANDARD_EVENTS_IDENTIFIER {
                supported_features.events = true;
            } else if feature.as_str() == SOLANA_SIGN_IN_IDENTIFIER {
                supported_features.sign_in = true;
            } else if feature.as_str() == SOLANA_SIGN_AND_SEND_TRANSACTION_IDENTIFIER {
                supported_features.sign_and_send_tx = true;
            } else if feature.as_str() == SOLANA_SIGN_TRANSACTION_IDENTIFIER {
                supported_features.sign_tx = true;
            } else if feature.as_str() == SOLANA_SIGN_MESSAGE_IDENTIFIER {
                supported_features.sign_message = true;
            } else {
                return Err(WalletError::UnsupportedWalletFeature(feature.to_owned()));
            }

            Ok(())
        })?;

        let icon = WalletIcon::from_jsvalue(&reflection)?;

        let label = match reflection.string("label") {
            Ok(value) => Some(value),
            Err(error) => match error {
                WalletError::InternalError(_) => Option::None,
                _ => {
                    return Err(error);
                }
            },
        };

        let account = WalletAccountData {
            address,
            public_key,
            chains,
            features,
            label,
            icon,
            supported_chains,
            supported_features,
        };

        Ok(Self {
            account,
            js_value: reflection.take(),
        })
    }

    /// Checks if MainNet is supported
    pub fn mainnet(&self) -> bool {
        self.account.supported_chains.mainnet
    }

    /// Checks if DevNet is supported
    pub fn devnet(&self) -> bool {
        self.account.supported_chains.devnet
    }

    /// Checks if TestNet is supported
    pub fn testnet(&self) -> bool {
        self.account.supported_chains.testnet
    }

    /// Checks if LocalNet is supported
    pub fn localnet(&self) -> bool {
        self.account.supported_chains.localnet
    }

    /// Checks if `standard:connect` is supported
    pub fn standard_connect(&self) -> bool {
        self.account.supported_features.connect
    }

    /// Checks if `standard:disconnect` is supported
    pub fn standard_disconnect(&self) -> bool {
        self.account.supported_features.disconnect
    }

    /// Checks if `standard:events` is supported
    pub fn standard_events(&self) -> bool {
        self.account.supported_features.events
    }

    /// Checks if `solana:signIn` is supported
    pub fn solana_signin(&self) -> bool {
        self.account.supported_features.sign_in
    }

    /// Checks if `solana:signMessage` is supported
    pub fn solana_sign_message(&self) -> bool {
        self.account.supported_features.sign_message
    }

    /// Checks if `solana:signAndSendTransaction` is supported
    pub fn solana_sign_and_send_transaction(&self) -> bool {
        self.account.supported_features.sign_and_send_tx
    }

    /// Checks if `solana:signTransaction` is supported
    pub fn solana_sign_transaction(&self) -> bool {
        self.account.supported_features.sign_tx
    }
}

impl core::fmt::Debug for WalletAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletAccount")
            .field("address", &self.account.address)
            .field("public_key", &self.account.public_key)
            .field("chains", &self.account.chains)
            .field("features", &self.account.features)
            .field("label", &self.account.label)
            .field("icon", &self.account.icon)
            .finish()
    }
}

impl PartialOrd for WalletAccount {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WalletAccount {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let inner_self: InnerWalletAccount = self.into();
        let inner_other: InnerWalletAccount = other.into();

        inner_self.cmp(&inner_other)
    }
}

impl core::cmp::Eq for WalletAccount {}

impl core::hash::Hash for WalletAccount {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let inner_self: InnerWalletAccount = self.into();

        inner_self.hash(state);
    }
}

// Reduce Eq, PartialEq, Ord, Hash work
#[derive(Eq, PartialEq, PartialOrd, Ord, Hash)]
struct InnerWalletAccount<'a> {
    pub address: &'a str,
    pub public_key: &'a [u8; 32],
    pub chains: &'a [String],
    pub features: &'a [String],
    pub label: Option<&'a String>,
    pub icon: Option<&'a String>,
}

impl<'a> From<&'a WalletAccount> for InnerWalletAccount<'a> {
    fn from(value: &'a WalletAccount) -> Self {
        Self {
            address: value.account.address.as_str(),
            public_key: &value.account.public_key,
            chains: value.account.chains.as_slice(),
            features: &value.account.features,
            label: value.account.label.as_ref(),
            icon: value.account.icon.as_ref(),
        }
    }
}
