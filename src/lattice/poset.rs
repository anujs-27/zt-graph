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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn test_flows_to_allow() {
        let source: SecurityLabel = SecurityLabel::new(Clearance::Public, BTreeSet::new());
        let mut target: SecurityLabel = SecurityLabel::new(Clearance::Restricted, BTreeSet::new());
        target.compartments.insert(String::from("HR"));
        assert!(
            flows_to(&source, &target),
            "[FAILED] Expected Public -> Restricted to be a valid flow."
        );
    }

    #[test]
    fn test_flows_to_reject() {
        let source: SecurityLabel = SecurityLabel::new(Clearance::Restricted, BTreeSet::new());
        let mut target: SecurityLabel = SecurityLabel::new(Clearance::Public, BTreeSet::new());
        target.compartments.insert(String::from("HR"));
        assert!(
            !flows_to(&source, &target),
            "[FAILED] Expected Restricted -> Public to be an invalid flow."
        );
    }

    #[test]
    fn test_is_incomparable() {
        let label_a =
            SecurityLabel::new(Clearance::Restricted, BTreeSet::from([String::from("HR")]));
        let label_b = SecurityLabel::new(
            Clearance::Restricted,
            BTreeSet::from([String::from("Finance")]),
        );

        assert!(
            is_incomparable(&label_a, &label_b),
            "[FAILED] Expected labels with disjoint compartments at the same clearance to be incomparable."
        );
    }

    #[test]
    fn test_is_covered_by_compartments() {
        let lower = SecurityLabel::new(Clearance::Public, BTreeSet::new());
        let upper = SecurityLabel::new(Clearance::Public, BTreeSet::from([String::from("HR")]));

        assert!(
            is_covered_by(&upper, &lower),
            "[FAILED] Expected upper to immediately cover lower due to compartment addition."
        );
    }

    #[test]
    fn test_is_covered_by_clearance() {
        let lower: SecurityLabel =
            SecurityLabel::new(Clearance::Public, BTreeSet::from([String::from("HR")]));
        let upper: SecurityLabel =
            SecurityLabel::new(Clearance::Restricted, BTreeSet::from([String::from("HR")]));

        assert!(
            is_covered_by(&upper, &lower),
            "[FAILED] Expected upper to immediately cover lower due to clearance addition."
        )
    }

    #[test]
    fn test_is_covered_by_assumes_flow() {
        let lower: SecurityLabel = SecurityLabel::new(Clearance::Public, BTreeSet::new());
        let upper_clearance_jump: SecurityLabel =
            SecurityLabel::new(Clearance::Confidential, BTreeSet::new());
        let upper_compartment_jump: SecurityLabel = SecurityLabel::new(
            Clearance::Public,
            BTreeSet::from([String::from("HR"), String::from("Engineering")]),
        );
        assert!(
            !is_covered_by(&upper_clearance_jump, &lower),
            "[FAILED] Wrong clearance level jump"
        );
        assert!(
            !is_covered_by(&upper_compartment_jump, &lower),
            "[FAILED] Wrong compartment level jump"
        )
    }
}
