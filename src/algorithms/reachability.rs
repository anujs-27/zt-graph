/*
[
 [a, b, c],
 [d, e, f],   -> [a, b, c, d, e, f, g, h, i]
 [g, h, i]
]
*/

use crate::{graph::topology::TopologyGraph, lattice::poset::flows_to};

pub struct ReachabilityMatrix {
    pub size: usize,
    pub matrix: Vec<bool>,
}

impl ReachabilityMatrix {
    pub fn new(size: usize) -> Self {
        ReachabilityMatrix {
            size,
            matrix: vec![false; size * size],
        }
    }

    pub fn get(&self, row: usize, col: usize) -> bool {
        let idx = row * self.size + col;
        self.matrix[idx]
    }

    pub fn set(&mut self, row: usize, col: usize, val: bool) {
        let idx = row * self.size + col;
        self.matrix[idx] = val;
    }

    pub fn is_reachable(&self, u: usize, v: usize) -> bool {
        if u >= self.size || v >= self.size {
            return false;
        }
        self.get(u, v)
    }

    pub fn compute_transitive_closure(adjacency_matrix: &[Vec<bool>]) -> Self {
        let size = adjacency_matrix.len();
        let mut matrix = ReachabilityMatrix::new(size);

        for i in 0..size {
            for j in 0..size {
                matrix.set(i, j, adjacency_matrix[i][j]);
            }
        }

        // Warshall's algorithm
        for k in 0..size {
            for i in 0..size {
                if matrix.get(i, k) {
                    for j in 0..size {
                        if !matrix.get(i, j) && matrix.get(k, j) {
                            matrix.set(i, j, true);
                        }
                    }
                }
            }
        }

        matrix
    }
}

pub fn audit_lattice_reachability(
    closure: &ReachabilityMatrix,
    topo: &TopologyGraph,
) -> Vec<(usize, usize)> {
    assert_eq!(
        closure.size,
        topo.zone_count(),
        "ReachabilityMatrix size must match TopologyGraph zone count"
    );

    let mut reachables: Vec<(usize, usize)> = Vec::new();

    for i in 0..closure.size {
        for j in 0..closure.size {
            if i == j {
                continue;
            }
            let label_u = &topo
                .get_zone(i)
                .expect("ERROR: Couldn't find the zone")
                .label;
            let label_v = &topo
                .get_zone(j)
                .expect("ERROR: Couldn't find the zone")
                .label;
            if closure.is_reachable(i, j) && !flows_to(label_u, label_v) {
                reachables.push((i, j));
            }
        }
    }

    reachables
}

/*
    TODO: Replace Warshall's algorithm with a targeted BFS for blast-radius calculation.
    Note: Warshall's is currently used to satisfy assignment rubric constraints (all-pairs reachability),
    but its O(V^3) time complexity is inefficient for large dependency graphs. - 2026-09-25
*/
