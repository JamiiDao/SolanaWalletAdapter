/// Solana Mainnet cluster
pub const MAINNET_ENDPOINT: &str = "https://api.mainnet-beta.solana.com";
/// Solana Devnet cluster
pub const DEVNET_ENDPOINT: &str = "https://api.devnet.solana.com";
/// Solana Testnet cluster
pub const TESTNET_ENDPOINT: &str = "https://api.testnet.solana.com";
/// Solana Localnet cluster
pub const LOCALNET_ENDPOINT: &str = "http://localhost:8899";

/// Solana Mainnet cluster,  [https://api.mainnet-beta.solana.com](https://api.mainnet-beta.solana.com)
pub const MAINNET_IDENTIFIER: &str = "solana:mainnet";
/// Solana Devnet cluster, e.g. [https://api.devnet.solana.com](https://api.devnet.solana.com)
pub const DEVNET_IDENTIFIER: &str = "solana:devnet";
/// Solana Testnet cluster, e.g. [https://api.testnet.solana.com](https://api.testnet.solana.com)
pub const TESTNET_IDENTIFIER: &str = "solana:testnet";
/// Solana Localnet cluster, e.g. [http://localhost:8899](http://localhost:8899)
pub const LOCALNET_IDENTIFIER: &str = "solana:localnet";

/// Solana Mainnet cluster,  [https://api.mainnet-beta.solana.com](https://api.mainnet-beta.solana.com)
pub const MAINNET: &str = "mainnet";
/// Solana Devnet cluster, e.g. [https://api.devnet.solana.com](https://api.devnet.solana.com)
pub const DEVNET: &str = "devnet";
/// Solana Testnet cluster, e.g. [https://api.testnet.solana.com](https://api.testnet.solana.com)
pub const TESTNET: &str = "testnet";
/// Solana Localnet cluster, e.g. [http://localhost:8899](http://localhost:8899)
pub const LOCALNET: &str = "localnet";

/// Solana Clusters
#[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Cluster {
    /// Solana Mainnet cluster,  [https://api.mainnet-beta.solana.com](https://api.mainnet-beta.solana.com)
    MainNet,
    /// Solana Devnet cluster, e.g. [https://api.devnet.solana.com](https://api.devnet.solana.com)
    #[default]
    DevNet,
    /// Solana Testnet cluster, e.g. [https://api.testnet.solana.com](https://api.testnet.solana.com)
    TestNet,
    /// Solana Localnet cluster, e.g. [http://localhost:8899](http://localhost:8899)
    LocalNet,
}

impl Cluster {
    /// A Solana endpoint URI
    pub fn endpoint(&self) -> &str {
        match self {
            Cluster::MainNet => MAINNET_ENDPOINT,
            Cluster::DevNet => DEVNET_ENDPOINT,
            Cluster::TestNet => TESTNET_ENDPOINT,
            Cluster::LocalNet => LOCALNET_ENDPOINT,
        }
    }

    /// A Solana cluster identifier
    pub fn chain(&self) -> &str {
        match self {
            Cluster::MainNet => MAINNET_IDENTIFIER,
            Cluster::DevNet => DEVNET_IDENTIFIER,
            Cluster::TestNet => TESTNET_IDENTIFIER,
            Cluster::LocalNet => LOCALNET_IDENTIFIER,
        }
    }

    /// A Solana cluster identifier as a &str
    pub fn display(&self) -> &str {
        match self {
            Cluster::MainNet => MAINNET,
            Cluster::DevNet => DEVNET,
            Cluster::TestNet => TESTNET,
            Cluster::LocalNet => LOCALNET,
        }
    }
}

impl core::fmt::Display for Cluster {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.display())
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
            MAINNET => Self::MainNet,
            DEVNET => Self::DevNet,
            TESTNET => Self::TestNet,
            LOCALNET => Self::LocalNet,
            _ => Self::DevNet,
        }
    }
}
