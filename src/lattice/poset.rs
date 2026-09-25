use crate::lattice::labels::{Clearance, SecurityLabel};

pub fn flows_to(source: &SecurityLabel, target: &SecurityLabel) -> bool {
    source.clearance <= target.clearance && source.compartments.is_subset(&target.compartments)
}

pub fn is_incomparable(a: &SecurityLabel, b: &SecurityLabel) -> bool {
    !flows_to(a, b) && !flows_to(b, a)
}

fn is_immediate_successor(a: Clearance, b: Clearance) -> bool {
    (a as usize) + 1 == (b as usize)
}

pub fn is_covered_by_assumes_flow(upper: &SecurityLabel, lower: &SecurityLabel) -> bool {
    if lower == upper {
        return false;
    }

    if lower.clearance == upper.clearance
        && upper.compartments.len() == lower.compartments.len() + 1
    {
        return true;
    }

    if lower.compartments == upper.compartments
        && is_immediate_successor(lower.clearance, upper.clearance)
    {
        return true;
    }

    false
}

pub fn is_covered_by(upper: &SecurityLabel, lower: &SecurityLabel) -> bool {
    flows_to(lower, upper) && is_covered_by_assumes_flow(upper, lower)
}
