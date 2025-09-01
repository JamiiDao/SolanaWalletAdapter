use std::borrow::Cow;

use base64ct::{Base64, Encoding};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct WalletStandardIcon {
    bytes: &'static [u8],
    mime: WalletStandardIconMime,
}

impl WalletStandardIcon {
    pub fn base64<'wa>(&'wa self) -> Cow<'wa, str> {
        let encoded = Base64::encode_string(self.bytes);

        Cow::Borrowed("data:image/") + self.mime.mime_str() + ";base64," + Cow::Owned(encoded)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum WalletStandardIconMime {
    Svg,
    Png,
    Webp,
    Gif,
}

impl WalletStandardIconMime {
    pub fn mime_str(&self) -> &str {
        match self {
            Self::Svg => "svg+xml",
            Self::Gif => "gif",
            Self::Png => "png",
            Self::Webp => "webp",
        }
    }
}
