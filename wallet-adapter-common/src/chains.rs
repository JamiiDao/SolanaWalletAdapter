/// Used as a helper struct to contain all the chains supported by a wallet
/// as defined by the wallet standard
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct ChainSupport {
    /// Main Net cluster
    pub mainnet: bool,
    /// Dev Net cluster
    pub devnet: bool,
    /// Test Net cluster
    pub testnet: bool,
    /// Local Net cluster
    pub localnet: bool,
}
