/// Solana Mainnet cluster,  https://api.mainnet-beta.solana.com
pub const MAINNET_IDENTIFIER: &'static str = "solana:mainnet";
/// Solana Devnet cluster, e.g. https://api.devnet.solana.com
pub const DEVNET_IDENTIFIER: &'static str = "solana:devnet";
/// Solana Testnet cluster, e.g. https://api.testnet.solana.com
pub const TESTNET_IDENTIFIER: &'static str = "solana:testnet";
/// Solana Localnet cluster, e.g. http://localhost:8899
pub const LOCALNET_IDENTIFIER: &'static str = "solana:localnet";

/// Solana Mainnet cluster
pub const MAINNET_ENDPOINT: &'static str = "https://api.mainnet-beta.solana.com";
/// Solana Devnet cluster
pub const DEVNET_ENDPOINT: &'static str = "https://api.devnet.solana.com";
/// Solana Testnet cluster
pub const TESTNET_ENDPOINT: &'static str = "https://api.testnet.solana.com";
/// Solana Localnet cluster
pub const LOCALNET_ENDPOINT: &'static str = "http://localhost:8899";

/// Solana Clusters
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Cluster {
    /// Solana Mainnet cluster,  https://api.mainnet-beta.solana.com
    MainNet,
    /// Solana Devnet cluster, e.g. https://api.devnet.solana.com
    DevNet,
    /// Solana Testnet cluster, e.g. https://api.testnet.solana.com
    TestNet,
    /// Solana Localnet cluster, e.g. http://localhost:8899
    LocalNet,
    Custom(String),
}

impl Cluster {
    /// A Solana endpoint URI
    pub fn endpoint(&self) -> &str {
        match self {
            Cluster::MainNet => MAINNET_ENDPOINT,
            Cluster::DevNet => DEVNET_ENDPOINT,
            Cluster::TestNet => TESTNET_ENDPOINT,
            Cluster::LocalNet => LOCALNET_ENDPOINT,
            Cluster::Custom(uri) => uri.as_str(),
        }
    }
}

impl From<&str> for Cluster {
    fn from(value: &str) -> Self {
        match value {
            MAINNET_IDENTIFIER => Self::MainNet,
            DEVNET_IDENTIFIER => Self::DevNet,
            TESTNET_IDENTIFIER => Self::TestNet,
            LOCALNET_IDENTIFIER => Self::LocalNet,
            MAINNET_ENDPOINT => Self::MainNet,
            DEVNET_ENDPOINT => Self::DevNet,
            TESTNET_ENDPOINT => Self::TestNet,
            LOCALNET_ENDPOINT => Self::LocalNet,
            _ => Self::Custom(value.to_string()),
        }
    }
}

#[cfg(test)]
mod chain_tests {
    use super::*;

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
        assert_eq!(Cluster::MainNet, "solana:mainnet".try_into().unwrap());
        assert_eq!(Cluster::DevNet, "solana:devnet".try_into().unwrap());
        assert_eq!(Cluster::TestNet, "solana:testnet".try_into().unwrap());
        assert_eq!(Cluster::LocalNet, "solana:localnet".try_into().unwrap());
        assert!({
            let chain: Result<Cluster, _> = "solana:localnet2".try_into();

            chain.is_err()
        });

        assert_eq!(
            Cluster::MainNet,
            "https://api.mainnet-beta.solana.com".try_into().unwrap()
        );
        assert_eq!(
            Cluster::DevNet,
            "https://api.devnet.solana.com".try_into().unwrap()
        );
        assert_eq!(
            Cluster::TestNet,
            "https://api.testnet.solana.com".try_into().unwrap()
        );
        assert_eq!(
            Cluster::LocalNet,
            "http://localhost:8899".try_into().unwrap()
        );
        assert!({
            let chain: Result<Cluster, _> = "https://localhost:8899".try_into();

            chain.is_err()
        });
        assert!({
            let chain: Result<Cluster, _> = "https://cluster.foo".try_into();

            chain.is_err()
        });
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
