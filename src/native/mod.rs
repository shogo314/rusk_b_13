//! Shared extern ABI, trusted effect contracts, and lowering descriptors.
pub(crate) mod abi;
pub(crate) mod effects;
pub(crate) mod lowering;

use std::collections::BTreeSet;

/// Native declarations and adapters requested while checking and specializing Rusk code.
#[derive(Clone, Default)]
pub(crate) struct NativeRequests {
    pub(crate) direct: BTreeSet<(usize, bool)>,
    pub(crate) declarations: BTreeSet<usize>,
    pub(crate) values: BTreeSet<usize>,
    pub(crate) owned: BTreeSet<usize>,
    pub(crate) rooted: BTreeSet<usize>,
    pub(crate) effects: BTreeSet<(usize, crate::analysis::effects::Query)>,
    pub(crate) unit_bridges: BTreeSet<(usize, abi::UnitBridge)>,
}

impl NativeRequests {
    pub(crate) fn targets(&self) -> impl Iterator<Item = usize> + '_ {
        self.direct
            .iter()
            .map(|&(id, _)| id)
            .chain(self.declarations.iter().copied())
            .chain(self.values.iter().copied())
            .chain(self.owned.iter().copied())
            .chain(self.rooted.iter().copied())
            .chain(self.unit_bridges.iter().map(|&(id, _)| id))
    }

    pub(crate) fn extend(&mut self, other: &Self) {
        self.direct.extend(&other.direct);
        self.declarations.extend(&other.declarations);
        self.values.extend(&other.values);
        self.owned.extend(&other.owned);
        self.rooted.extend(&other.rooted);
        self.effects.extend(&other.effects);
        self.unit_bridges.extend(&other.unit_bridges);
    }
}
