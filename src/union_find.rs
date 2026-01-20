use std::collections::HashMap;

// partitions a set into disjoint subsets
// every subset is represented by a node
pub struct UnionFind {
    // stores sets as trees, every element
    // contains index of parent node.
    // roots contain their own index
    nodes: Vec<Node>,
}

#[derive(Clone, PartialEq, Debug)]
struct Node {
    // index of the parent, if this points
    // to itself its a root
    parent: usize,
    // number of descendents,
    // only valid for root nodes
    size: usize,
}

// being generic in the index sucks
// impl<I: Into<usize> + From<usize> + Copy + PartialEq + PartialOrd + AddAssign> UnionFind<I> {
impl UnionFind {
    pub fn new(size: usize) -> Self {
        UnionFind {
            nodes: (0..)
                .take(size)
                .map(|i| Node { parent: i, size: 1 })
                .collect(),
        }
    }

    pub fn find(&mut self, x: usize) -> usize {
        let node = &self.nodes[x];
        if node.parent != x {
            self.nodes[x].parent = self.find(node.parent);
            return self.nodes[x].parent;
        }
        x
    }

    pub fn union(&mut self, x: usize, y: usize) {
        let root_x_idx = self.find(x);
        let root_y_idx = self.find(y);

        if root_x_idx == root_y_idx {
            return;
        }

        // to prevent trees from becoming too deep, make sure to add the smaller tree to the larger
        let (smaller_idx, larger_idx) = {
            if self.nodes[root_x_idx].size < self.nodes[root_y_idx].size {
                (root_x_idx, root_y_idx)
            } else {
                (root_y_idx, root_x_idx)
            }
        };
        // minmax_by_key(root_x_idx, root_y_idx, |&idx| self.nodes[idx].size);

        self.nodes[smaller_idx].parent = larger_idx;
        self.nodes[larger_idx].size += self.nodes[smaller_idx].size;
    }

    pub fn get_sets(mut self) -> impl Iterator<Item = Vec<usize>> {
        let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();

        for i in 0..self.nodes.len() {
            let root = self.find(i);
            groups.entry(root).or_insert_with(Vec::new).push(i);
        }

        groups.into_values()
    }
}

// thanks Claude
#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_find_root() {
        let mut uf = UnionFind::new(3);
        assert_eq!(uf.find(0), 0);
        assert_eq!(uf.find(1), 1);
        assert_eq!(uf.find(2), 2);
    }

    #[test]
    fn test_union_basic() {
        let mut uf = UnionFind::new(4);
        uf.union(0, 1);
        assert_eq!(uf.find(0), uf.find(1));
        assert_ne!(uf.find(0), uf.find(2));
    }

    #[test]
    fn test_union_multiple() {
        let mut uf = UnionFind::new(5);
        uf.union(0, 1);
        uf.union(1, 2);
        uf.union(3, 4);

        assert_eq!(uf.find(0), uf.find(1));
        assert_eq!(uf.find(1), uf.find(2));
        assert_eq!(uf.find(3), uf.find(4));
        assert_ne!(uf.find(0), uf.find(3));
    }

    #[test]
    fn test_union_same_element() {
        let mut uf = UnionFind::new(3);
        uf.union(1, 1);
        assert_eq!(uf.find(1), 1);
    }

    #[test]
    fn test_path_compression() {
        let mut uf = UnionFind::new(4);
        uf.union(0, 1);
        uf.union(1, 2);
        uf.union(2, 3);

        // First find should trigger path compression
        let root = uf.find(3);
        // All nodes should now point directly to root
        assert_eq!(uf.nodes[0].parent, root);
        assert_eq!(uf.nodes[1].parent, root);
        assert_eq!(uf.nodes[2].parent, root);
        assert_eq!(uf.nodes[3].parent, root);
    }

    #[test]
    fn test_disjoint_sets() {
        let mut uf = UnionFind::new(6);
        uf.union(0, 1);
        uf.union(2, 3);
        uf.union(4, 5);

        // Three separate components
        assert_ne!(uf.find(0), uf.find(2));
        assert_ne!(uf.find(0), uf.find(4));
        assert_ne!(uf.find(2), uf.find(4));
    }

    #[test]
    fn test_union_find_into_sets() {
        let mut uf = UnionFind::new(6);

        // Create some unions: {0,1,2}, {3,4}, {5}
        uf.union(0, 1);
        uf.union(1, 2);
        uf.union(3, 4);

        // let sets: Vec<Vec<usize>> = .collect();

        // // Should have 3 sets
        // assert_eq!(sets.len(), 3);

        // Check that we have the expected sets (order doesn't matter)
        // let mut found_sets = Vec::new();
        // for set in sets {
        //     found_sets.push(set);
        // }

        // // Sort by size for consistent testing
        // found_sets.sort_by_key(|s| s.len());

        // assert_eq!(found_sets[0], HashSet::from([5])); // singleton
        // assert_eq!(found_sets[1], HashSet::from([3, 4])); // pair
        // assert_eq!(found_sets[2], HashSet::from([0, 1, 2])); // triple

        let found_sets: HashMap<usize, HashSet<usize>> = uf
            .get_sets()
            .map(|set| (set.len(), set.into_iter().collect()))
            .collect();

        assert_eq!(
            found_sets,
            HashMap::from([
                (1, HashSet::from([5])),       // singleton
                (2, HashSet::from([3, 4])),    // pair
                (3, HashSet::from([0, 1, 2])), // triple
            ])
        );
    }
}
