pub type Byte32Array = [u8; 32];

pub type Byte64Array = [u8; 64];

pub struct BaseUtils;

impl BaseUtils {
    /// Converts [SystemTime] to ISO 8601 datetime string as required by
    /// Sign In With for wallet-standard
    #[cfg(feature = "signIn")]
    pub fn to_iso860(system_time: std::time::SystemTime) -> humantime::Rfc3339Timestamp {
        humantime::format_rfc3339_millis(system_time)
    }
}
