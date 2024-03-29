use crate::Solid;
use cgmath::{prelude::*, Vector3};
use std::iter::repeat;

/// Build a binary axis-aligned bounding box tree in `O(n^2)` where `n = solids.len()`
// TODO: Mark root-ness externally and allow combining nodes/final_tree
pub fn build_tree(solids: &[Solid]) -> Vec<Node> {
    let mut nodes: Vec<Option<Node>> = solids
        .iter()
        .enumerate()
        .map(Node::leaf)
        .map(|node| Some(node))
        .collect();

    // Every node is initially a root. When joining two spheres two roots are removed and one added.
    let mut num_roots = nodes.len();

    let tot_nodes = 2 * nodes.len() - 1;
    nodes.reserve_exact(nodes.len() - 1);

    // Non-root nodes are removed from [nodes] and put in [final_tree]. This is necessary for us to
    // push a currently-root node into the empty chain and to iterate through only currently-root
    // nodes when searching for the nearest neighbor. TODO make unnecesary by external tagging
    let mut final_tree: Vec<Node> = repeat(Node::placeholder()).take(tot_nodes).collect();

    let mut chain: Vec<usize> = Vec::new(); // Every node has the next one as its nearest neighbor

    while num_roots > 1 {
        let current = loop {
            if chain.is_empty() {
                // Put arbitrary node on empty stack
                chain.push(
                    nodes
                        .iter()
                        .enumerate()
                        .rev() // Pushing nodes from last to first is better?
                        .find(|(_, s)| Option::is_some(*s))
                        .unwrap()
                        .0,
                );
            }
            let current = *chain.last().unwrap();
            if nodes[current].is_some() {
                break current;
            }
            chain.pop();
        };
        // Find closest neighbor
        let (_cost, nearest_neighbor) = nodes
            .iter()
            .enumerate()
            .filter(|(i, neighbor)| *i != current && neighbor.is_some())
            .map(|(i, neighbor)| (metric(&nodes[current].unwrap(), &neighbor.unwrap()), i))
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        if chain.len() >= 2 && nearest_neighbor == chain[chain.len() - 2] {
            // Join a pair of mutually closest neighbors
            let last = nearest_neighbor;
            nodes.push(Some(Node::branch(current, last, &nodes)));
            final_tree[current] = nodes[current].take().unwrap();
            final_tree[last] = nodes[last].take().unwrap();
            num_roots -= 1;
            chain.pop();
            chain.pop();
        } else {
            // Found closer pair, pushing to stack
            chain.push(nearest_neighbor);
        }
    }
    final_tree[tot_nodes - 1] = nodes.last().unwrap().unwrap(); // Push root

    // Reverse the entire tree, so that the shader can find the root node at index 0
    // This is necessary as the size of the tree is dynamic
    final_tree.reverse();
    final_tree
        .iter_mut()
        .for_each(|node| node.reflect_child_indices(tot_nodes - 1));
    final_tree
}

// This is not strictly a valid metric for the nearest-neighbor chain algorithm,
// but we are satisfied with an approximate tree.
// This can be thought of as the "cost" of joining two nodes.
fn metric(a: &Node, b: &Node) -> f32 {
    let a_extent = a.max - a.min;
    let b_extent = b.max - b.min;
    let c_extent = Vector3::new(
        f32::max(a.max.x, b.max.x) - f32::min(a.min.x, b.min.x),
        f32::max(a.max.y, b.max.y) - f32::min(a.min.y, b.min.y),
        f32::max(a.max.z, b.max.z) - f32::min(a.min.z, b.min.z),
    );
    // We return the increase in total surface area (up to a constant factor) after
    // a join.
    return area(c_extent) - area(a_extent) - area(b_extent);

    fn area(ext: Vector3<f32>) -> f32 {
        ext.x * ext.y + ext.y * ext.z + ext.z * ext.x
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Node {
    min: Vector3<f32>,
    left: u32,

    max: Vector3<f32>,
    right: u32,
}

unsafe impl bytemuck::Pod for Node {}
unsafe impl bytemuck::Zeroable for Node {}

impl Node {
    const NO_RIGHT_CHILD: u32 = u32::MAX;
    // Takes a tuple as it will be produced by [Iterator::enumerate]
    pub(self) fn leaf((index, solid): (usize, &Solid)) -> Self {
        let (min, max) = solid.bounding_aabb();
        Self {
            min,
            max,
            left: index as u32,
            right: Self::NO_RIGHT_CHILD,
        }
    }
    pub(self) fn branch(a_index: usize, b_index: usize, nodes: &Vec<Option<Node>>) -> Self {
        let a = nodes[a_index].unwrap();
        let b = nodes[b_index].unwrap();
        Self {
            min: Vector3::new(
                f32::min(a.min.x, b.min.x),
                f32::min(a.min.y, b.min.y),
                f32::min(a.min.z, b.min.z),
            ),
            max: Vector3::new(
                f32::max(a.max.x, b.max.x),
                f32::max(a.max.y, b.max.y),
                f32::max(a.max.z, b.max.z),
            ),
            left: a_index as u32,
            right: b_index as u32,
        }
    }
    pub(self) fn reflect_child_indices(&mut self, last_index: usize) {
        // Only if we are a branch
        if self.right != Self::NO_RIGHT_CHILD {
            self.left = (last_index as u32) - self.left;
            self.right = (last_index as u32) - self.right;
        }
    }
    pub(self) fn placeholder() -> Self {
        Self {
            min: Vector3::zero(),
            max: Vector3::zero(),
            left: Self::NO_RIGHT_CHILD,
            right: Self::NO_RIGHT_CHILD,
        }
    }
}
