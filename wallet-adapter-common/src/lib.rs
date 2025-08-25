#![forbid(unsafe_code)]
#![forbid(missing_docs)]

//! Common utilities to identify features and chains in the `wallet-adapter` standard.

mod errors;
pub use errors::*;

mod wallet_account;
pub use wallet_account::*;

mod wallet;
pub use wallet::*;

mod version;
pub use version::*;

mod utils;
pub use utils::*;

/// Feature support struct
pub mod feature_support;

/// The Solana identifiers for standardized events of the `wallet-adapter`
pub mod standardized_events;

/// The Solana signin standard `solana:signIn` the `wallet-adapter`
pub mod signin_standard;

/// Supported `chains` of the Solana `wallet-adapter` standard
pub mod chains;

/// Cluster identifiers for Solana `wallet-adapter` standard
pub mod clusters;

#[cfg(test)]
mod chain_tests {
    use super::clusters::*;

    #[test]
    fn is_valid_uri() {
        assert_eq!(MAINNET_ENDPOINT, "https://api.mainnet-beta.solana.com");
        assert_eq!(DEVNET_ENDPOINT, "https://api.devnet.solana.com");
        assert_eq!(TESTNET_ENDPOINT, "https://api.testnet.solana.com");
        assert_eq!(LOCALNET_ENDPOINT, "http://localhost:8899");

        assert_eq!(MAINNET_IDENTIFIER, "solana:mainnet");
        assert_eq!(DEVNET_IDENTIFIER, "solana:devnet");
        assert_eq!(TESTNET_IDENTIFIER, "solana:testnet");
        assert_eq!(LOCALNET_IDENTIFIER, "solana:localnet");
    }

    #[test]
    fn valid_chain() {
        assert_eq!(Cluster::MainNet, "solana:mainnet".into());
        assert_eq!(Cluster::DevNet, "solana:devnet".into());
        assert_eq!(Cluster::TestNet, "solana:testnet".into());
        assert_eq!(Cluster::LocalNet, "solana:localnet".into());
        assert!(Cluster::DevNet == "solana:localnet2".into());

        assert_eq!(
            Cluster::MainNet,
            "https://api.mainnet-beta.solana.com".into()
        );
        assert_eq!(Cluster::DevNet, "https://api.devnet.solana.com".into());
        assert_eq!(Cluster::TestNet, "https://api.testnet.solana.com".into());
        assert_eq!(Cluster::LocalNet, "http://localhost:8899".into());
        assert!(Cluster::DevNet == "https://localhost:8899".into());
        assert!(Cluster::DevNet == "https://cluster.foo".into());
    }

    #[test]
    fn validate_endpoint() {
        assert_eq!(
            Cluster::MainNet.endpoint(),
            "https://api.mainnet-beta.solana.com"
        );
        assert_eq!(Cluster::DevNet.endpoint(), "https://api.devnet.solana.com");
        assert_eq!(
            Cluster::TestNet.endpoint(),
            "https://api.testnet.solana.com"
        );
        assert_eq!(Cluster::LocalNet.endpoint(), "http://localhost:8899");
    }
}
