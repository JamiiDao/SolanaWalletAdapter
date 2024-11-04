use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Reflect;

use crate::{
    Cluster, Features, Reflection, SemverVersion, WalletAccount, WalletError, WalletIcon,
    WalletResult,
};

use super::ChainSupport;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Wallet {
    name: String,
    version: SemverVersion,
    icon: Option<WalletIcon>,
    accounts: Vec<WalletAccount>,
    chains: Vec<Cluster>,
    features: Features,
    // Convinience field, instead of iteration through the `chains` field
    supported_chains: ChainSupport,
}

impl Wallet {
    pub fn from_jsvalue(value: JsValue) -> WalletResult<Self> {
        let reflection = Reflection::new(value)?;

        let mut supported_chains = ChainSupport::default();

        let chains_raw = reflection.vec_string_and_filter("chains", "solana:")?;
        let chains = chains_raw
            .into_iter()
            .map(|chain_raw| {
                let cluster = chain_raw.as_str().try_into();
                if let Ok(cluster_inner) = &cluster {
                    if cluster_inner == &Cluster::MainNet {
                        supported_chains.mainnet = true;
                    } else if cluster_inner == &Cluster::DevNet {
                        supported_chains.devnet = true;
                    } else if cluster_inner == &Cluster::TestNet {
                        supported_chains.testnet = true;
                    } else if cluster_inner == &Cluster::LocalNet {
                        supported_chains.localnet = true;
                    } else {
                    }
                }

                cluster
            })
            .collect::<WalletResult<Vec<Cluster>>>()?;

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
            supported_chains,
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

    pub fn features(&self) -> &Features {
        &self.features
    }

    pub fn accounts(&self) -> &[WalletAccount] {
        &self.accounts
    }

    pub fn chains(&self) -> &[Cluster] {
        &self.chains
    }

    pub fn mainnet(&self) -> bool {
        self.supported_chains.mainnet
    }

    pub fn devnet(&self) -> bool {
        self.supported_chains.devnet
    }

    pub fn testnet(&self) -> bool {
        self.supported_chains.testnet
    }

    pub fn localnet(&self) -> bool {
        self.supported_chains.localnet
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
