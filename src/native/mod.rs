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
