use super::*;

/// Normalize directed acyclic graph such that all children are sorted in memory,
/// and no child is stored before its parent.
pub fn fix(nodes: &mut [Node]) {
    tree_mem_sort::sort(nodes, |n| &mut n.parent, |n| &mut n.children)
}
