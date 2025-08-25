use crate::WalletAccountData;

/// The output of Sign In With Solana (SIWS) response from a wallet
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct SignInOutput {
    /// A [An Account](WalletAccountData)
    pub account: WalletAccountData,
    /// The UTF-8 encoded message
    pub message: String,
    /// The signature as a  byte array of 64 bytes in length corresponding to a
    /// [Ed25519 Signature](ed25519_dalek::Signature)
    pub signature: [u8; 64],
    /// The public key as a  byte array of 32 bytes in length corresponding to a
    /// [Ed25519 Public Key](ed25519_dalek::VerifyingKey)
    pub public_key: [u8; 32],
}

impl SignInOutput {
    /// Base58 encoded signature
    pub fn signature(&self) -> String {
        bs58::encode(&self.signature).into_string()
    }

    /// Base58 encoded [Ed25519 Public Key](ed25519_dalek::VerifyingKey)
    pub fn public_key(&self) -> String {
        bs58::encode(&self.public_key).into_string()
    }
    /// The address of the account the wallet used to sign the message
    pub fn address(&self) -> &str {
        self.account.address.as_str()
    }
}
