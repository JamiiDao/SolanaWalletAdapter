use std::borrow::Cow;

/// The Version of the Wallet Standard currently implemented.
/// This may be used by the app to determine compatibility and feature detect.
pub const WALLET_STANDARD_VERSION: &str = "1.0.0";

/// Semver Versioning struct
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemverVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

impl SemverVersion {
    /// Instantiate a new [SemverVersion]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the major version
    pub fn set_major(mut self, major: u8) -> Self {
        self.major = major;

        self
    }
    /// Set the minor version
    pub fn set_minor(mut self, minor: u8) -> Self {
        self.minor = minor;

        self
    }

    /// Set the patch version
    pub fn set_patch(mut self, patch: u8) -> Self {
        self.patch = patch;

        self
    }

    /// The major version
    pub fn major(&self) -> u8 {
        self.major
    }

    /// The minor version
    pub fn minor(&self) -> u8 {
        self.minor
    }

    /// The patch version
    pub fn patch(&self) -> u8 {
        self.patch
    }

    /// Get the string version of [Self] in the format `major.minor.patch`
    pub fn stringify_version<'a>(&'a self) -> Cow<'a, str> {
        Cow::Borrowed("")
            + Cow::Owned(self.major.to_string())
            + "."
            + Cow::Owned(self.minor.to_string())
            + "."
            + Cow::Owned(self.minor.to_string())
    }
}

impl core::fmt::Debug for SemverVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SemverVersion({}.{}.{})",
            self.major, self.minor, self.patch
        )
    }
}

impl core::fmt::Display for SemverVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
