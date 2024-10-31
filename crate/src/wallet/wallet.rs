use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Reflect;

use crate::{
    Features, Reflection, SemverVersion, WalletAccount, WalletError, WalletIcon, WalletResult,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Wallet {
    name: String,
    version: SemverVersion,
    icon: Option<WalletIcon>,
    accounts: Vec<WalletAccount>,
    chains: Vec<String>,
    features: Features,
}

impl Wallet {
    pub fn from_jsvalue(value: JsValue) -> WalletResult<Self> {
        let reflection = Reflection::new(value)?;

        let chains = reflection.vec_string_and_filter("chains", "solana:")?;

        let name = reflection.string("name")?;
        let version = SemverVersion::parse(&reflection.string("version")?)?;
        let icon = WalletIcon::from_jsvalue(&reflection)?;
        let accounts = Self::get_accounts(&reflection, "accounts")?;
        let features = Features::parse(&reflection)?;

        Ok(Wallet {
            name,
            version,
            icon,
            accounts,
            chains,
            features,
        })
    }

    pub fn get_accounts(reflection: &Reflection, key: &str) -> WalletResult<Vec<WalletAccount>> {
        let accounts_raw = Reflect::get(reflection.get_inner(), &key.into())?;
        Reflection::check_is_undefined(&accounts_raw)?;

        if !accounts_raw.is_array() {
            return Err(WalletError::ExpectedArray(
                "Reflection for `accounts` key".to_string(),
            ));
        }

        let accounts_array: js_sys::Array = accounts_raw.unchecked_into();

        accounts_array
            .iter()
            .map(|account| WalletAccount::parse(&Reflection::new(account)?))
            .collect::<WalletResult<Vec<WalletAccount>>>()
    }

    pub fn accounts(&self) -> &[WalletAccount] {
        &self.accounts
    }

    pub fn chains(&self) -> &[String] {
        &self.chains
    }

    pub fn icon(&self) -> Option<&WalletIcon> {
        self.icon.as_ref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &SemverVersion {
        &self.version
    }
}
