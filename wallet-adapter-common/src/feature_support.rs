/// Used as a helper struct to contain all the features supported by a wallet
/// as defined by the wallet standard
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct FeatureSupport {
    /// 'standard:connect'
    pub connect: bool,
    /// 'standard:disconnect'
    pub disconnect: bool,
    /// 'standard:events'
    pub events: bool,
    /// 'solana:signIn'
    pub sign_in: bool,
    /// 'solana:signMessage'
    pub sign_message: bool,
    /// 'solana:signAndSendTransaction'
    pub sign_and_send_tx: bool,
    /// 'solana:signTransaction'
    pub sign_tx: bool,
}
