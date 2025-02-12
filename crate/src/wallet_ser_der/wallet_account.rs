use std::borrow::Cow;

use web_sys::wasm_bindgen::JsValue;

use crate::{
    Reflection, Utils, WalletError, WalletIcon, WalletResult,
    SOLANA_SIGN_AND_SEND_TRANSACTION_IDENTIFIER, SOLANA_SIGN_IN_IDENTIFIER,
    SOLANA_SIGN_MESSAGE_IDENTIFIER, SOLANA_SIGN_TRANSACTION_IDENTIFIER,
    STANDARD_CONNECT_IDENTIFIER, STANDARD_DISCONNECT_IDENTIFIER, STANDARD_EVENTS_IDENTIFIER,
};

use super::{ChainSupport, Cluster, FeatureSupport};

/// Interface of a **WalletAccount**, also referred to as an **Account**.
/// An account is a _read-only data object_ that is provided from the Wallet to the app,
/// authorizing the app to use it.
/// The app can use an account to display and query information from a chain.
/// The app can also act using an account by passing it to `features` field of the Wallet.
#[derive(Clone, Default, PartialEq)]
pub struct WalletAccount {
    /// Address of the account, corresponding with a public key.
    pub address: String,
    /// Public key of the account, corresponding with a secret key to use.
    pub public_key: [u8; 32],
    /// Chains supported by the account.
    /// This must be a subset of the {@link Wallet.chains | chains} of the Wallet.
    pub chains: Vec<String>,
    /// Feature names supported by the account.
    /// This must be a subset of the names of {@link Wallet.features | features} of the Wallet.
    pub features: Vec<String>,
    /// Optional user-friendly descriptive label or name for the account. This may be displayed by the app.
    pub label: Option<String>,
    /// Optional user-friendly icon for the account. This may be displayed by the app. */
    pub icon: Option<WalletIcon>,
    /// The Javascript Value Representation of a wallet,
    /// this mostly used internally in the wallet adapter
    pub(crate) js_value: JsValue,
    // Convenience field, instead of going through the `features` field
    supported_features: FeatureSupport,
    // Convenience field, instead of iteration through the `chains` field
    supported_chains: ChainSupport,
}

impl core::fmt::Debug for WalletAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletAccount")
            .field("address", &self.address)
            .field("public_key", &self.public_key)
            .field("chains", &self.chains)
            .field("features", &self.features)
            .field("label", &self.label)
            .field("icon", &self.icon)
            .finish()
    }
}

impl WalletAccount {
    /// Get the shortened address of the `Base58 address` .
    /// It displays the first 4 characters and the last for characters
    /// separated by ellipsis eg `FXdl...RGd4` .
    /// If the address is less than 8 characters, an error is thrown
    pub fn shorten_address(&self) -> WalletResult<Cow<str>> {
        Utils::shorten_base58(&self.address)
    }

    /// Same as [Self::shorten_address] but with a custom range
    /// instead of taking the first 4 character and the last 4 characters
    /// it uses a custom range.
    pub fn custom_shorten_address(&self, take: usize) -> WalletResult<Cow<str>> {
        Utils::custom_shorten_base58(&self.address, take)
    }

    /// Same as [Self::shorten_address] but with a custom range
    /// instead of taking the first 4 character and the last 4 characters
    /// it uses a custom range for first characters before ellipsis and last characters after ellipsis.
    pub fn custom_shorten_address_rl(&self, left: usize, right: usize) -> WalletResult<Cow<str>> {
        if self.address.len() < left + right {
            return Err(WalletError::InvalidBase58Address);
        }

        let first_part = &self.address[..left];
        let last_part = &self.address[self.address.len() - right..];

        Ok(Cow::Borrowed(first_part) + "..." + last_part)
    }

    /// Parse A [WalletAccount] from [JsValue]
    pub(crate) fn parse(reflection: Reflection) -> WalletResult<Self> {
        let address = reflection.string("address")?;
        let public_key = reflection.byte32array("publicKey")?;
        let chains = reflection.vec_string("chains")?;
        let features = reflection.vec_string("features")?;

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

        Ok(Self {
            address,
            public_key,
            chains,
            features,
            label,
            icon,
            supported_chains,
            supported_features,
            js_value: reflection.take(),
        })
    }

    /// Checks if MainNet is supported
    pub fn mainnet(&self) -> bool {
        self.supported_chains.mainnet
    }

    /// Checks if DevNet is supported
    pub fn devnet(&self) -> bool {
        self.supported_chains.devnet
    }

    /// Checks if TestNet is supported
    pub fn testnet(&self) -> bool {
        self.supported_chains.testnet
    }

    /// Checks if LocalNet is supported
    pub fn localnet(&self) -> bool {
        self.supported_chains.localnet
    }

    /// An optional [WalletIcon]
    pub fn icon(&self) -> Option<&WalletIcon> {
        self.icon.as_ref()
    }

    /// Checks if `standard:connect` is supported
    pub fn standard_connect(&self) -> bool {
        self.supported_features.connect
    }

    /// Checks if `standard:disconnect` is supported
    pub fn standard_disconnect(&self) -> bool {
        self.supported_features.disconnect
    }

    /// Checks if `standard:events` is supported
    pub fn standard_events(&self) -> bool {
        self.supported_features.events
    }

    /// Checks if `solana:signIn` is supported
    pub fn solana_signin(&self) -> bool {
        self.supported_features.sign_in
    }

    /// Checks if `solana:signMessage` is supported
    pub fn solana_sign_message(&self) -> bool {
        self.supported_features.sign_message
    }

    /// Checks if `solana:signAndSendTransaction` is supported
    pub fn solana_sign_and_send_transaction(&self) -> bool {
        self.supported_features.sign_and_send_tx
    }

    /// Checks if `solana:signTransaction` is supported
    pub fn solana_sign_transaction(&self) -> bool {
        self.supported_features.sign_tx
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
    pub icon: Option<&'a WalletIcon>,
}

impl<'a> From<&'a WalletAccount> for InnerWalletAccount<'a> {
    fn from(value: &'a WalletAccount) -> Self {
        Self {
            address: value.address.as_str(),
            public_key: &value.public_key,
            chains: value.chains.as_slice(),
            features: &value.features,
            label: value.label.as_ref(),
            icon: value.icon.as_ref(),
        }
    }
}
