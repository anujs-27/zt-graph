use std::collections::BTreeSet;

use crate::lattice::labels::SecurityLabel;

pub fn flows_to(source: &SecurityLabel, target: &SecurityLabel) -> bool {
    source.clearance <= target.clearance && source.compartments.is_subset(&target.compartments)
}

pub fn is_incomparable(a: &SecurityLabel, b: &SecurityLabel) -> bool {
    !flows_to(a, b) && !flows_to(b, a)
}

pub fn least_upper_bound(a: &SecurityLabel, b: &SecurityLabel) -> SecurityLabel {
    let clearance = std::cmp::max(a.clearance, b.clearance);
    let union: BTreeSet<String> = a.compartments.union(&b.compartments).cloned().collect();
    SecurityLabel {
        clearance,
        compartments: union,
    }
}

pub fn greatest_lower_bound(a: &SecurityLabel, b: &SecurityLabel) -> SecurityLabel {
    let clearance = std::cmp::min(a.clearance, b.clearance);
    let intersection: BTreeSet<String> = a
        .compartments
        .intersection(&b.compartments)
        .cloned()
        .collect();

    SecurityLabel {
        clearance,
        compartments: intersection,
    }
}
