use core::{fmt, hash::Hash};

pub trait Cluster: fmt::Debug + PartialEq + Eq + PartialOrd + Ord + Hash + Clone + Copy {
    fn identifier(&self) -> &str;

    fn chain(&self) -> &str;

    fn endpoint(&self) -> &str;
}

pub trait ClusterEnabled {
    fn mainnet(&self) -> bool {
        true
    }

    fn testnet(&self) -> bool {
        true
    }

    fn devnet(&self) -> bool {
        true
    }

    fn localnet(&self) -> bool {
        true
    }
}
