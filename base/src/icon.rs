use std::borrow::Cow;

use base64ct::{Base64, Encoding};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct WalletStandardIcon {
    bytes: &'static [u8],
    mime: WalletStandardIconMime,
}

impl WalletStandardIcon {
    pub fn new(bytes: &'static [u8], mime: WalletStandardIconMime) -> Self {
        Self { bytes, mime }
    }

    pub fn new_svg(bytes: &'static [u8]) -> Self {
        Self::new(bytes, WalletStandardIconMime::Svg)
    }

    pub fn new_gif(bytes: &'static [u8]) -> Self {
        Self::new(bytes, WalletStandardIconMime::Gif)
    }

    pub fn new_webp(bytes: &'static [u8]) -> Self {
        Self::new(bytes, WalletStandardIconMime::Webp)
    }

    pub fn new_png(bytes: &'static [u8]) -> Self {
        Self::new(bytes, WalletStandardIconMime::Png)
    }

    pub fn new_jpeg(bytes: &'static [u8]) -> Self {
        Self::new(bytes, WalletStandardIconMime::Jpeg)
    }

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
    Jpeg,
}

impl WalletStandardIconMime {
    pub fn mime_str(&self) -> &str {
        match self {
            Self::Svg => "svg+xml",
            Self::Gif => "gif",
            Self::Png => "png",
            Self::Webp => "webp",
            Self::Jpeg => "jpeg",
        }
    }
}
