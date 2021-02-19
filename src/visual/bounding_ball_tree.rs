use crate::common::{Cuboid, Cylinder, Sphere};
use cgmath::{prelude::*, Vector3};
use std::iter::repeat;

/// Build a bounding ball hierarchy binary tree in `O(n^2)` where
/// `n = spheres.len() + cylinders.len() + cuboids.len()`
// TODO: Mark root-ness externally and allow combining nodes/final_tree
pub fn build_tree(spheres: &[Sphere], cylinders: &[Cylinder], cuboids: &[Cuboid]) -> Vec<Node> {
    let spheres = spheres.iter().enumerate().map(Node::leaf_sphere);
    let cylinders = cylinders.iter().enumerate().map(Node::leaf_cylinder);
    let cuboids = cuboids.iter().enumerate().map(Node::leaf_cuboid);

    let mut nodes: Vec<Option<Node>> = spheres
        .chain(cylinders)
        .chain(cuboids)
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

// This is not strictly a valid metric for the nearest-neighbor chain algorithm, but we are
// satisfied with an approximate tree
fn metric(a: &Node, b: &Node) -> f32 {
    let joined_radius = ((a.pos - b.pos).magnitude() + a.radius + b.radius) / 2.0;
    // We return the increase in total volume after a join
    let res = joined_radius.powi(3) - a.radius.powi(3) - b.radius.powi(3);
    if res.is_nan() {
        panic!("Bad nodes: {:?}, {:?}", a, b)
    } else {
        res
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Node {
    pos: Vector3<f32>,
    radius: f32,

    left: i32,
    right: u32,

    _padding: [u32; 2],
}

unsafe impl bytemuck::Pod for Node {}
unsafe impl bytemuck::Zeroable for Node {}

impl Node {
    // Takes a tuple as it will be produced by [Iterator::enumerate]
    pub(self) fn leaf_sphere((index, sphere): (usize, &Sphere)) -> Self {
        Self {
            pos: sphere.pos(),
            radius: sphere.radius(),
            left: -1, // Kind: Sphere => -1
            right: index as u32,
            _padding: [0; 2],
        }
    }
    // Takes a tuple as it will be produced by [Iterator::enumerate]
    pub(self) fn leaf_cylinder((index, cylinder): (usize, &Cylinder)) -> Self {
        todo!()
    }
    // Takes a tuple as it will be produced by [Iterator::enumerate]
    pub(self) fn leaf_cuboid((index, cuboid): (usize, &Cuboid)) -> Self {
        todo!()
    }
    pub(self) fn branch(a_index: usize, b_index: usize, nodes: &Vec<Option<Node>>) -> Self {
        let a = nodes[a_index].unwrap();
        let b = nodes[b_index].unwrap();
        let rel_pos_norm = if b.pos == a.pos {
            Vector3::zero()
        } else {
            (b.pos - a.pos).normalize()
        };
        let distance = (b.pos - a.pos).magnitude();
        let joined_midpoint =
            ((a.pos - rel_pos_norm * a.radius) + (b.pos + rel_pos_norm * b.radius)) / 2.0;
        let joined_radius = (distance + a.radius + b.radius) / 2.0;
        // The smallest sphere that encloses nodes [a] and [b]
        Self {
            pos: joined_midpoint,
            radius: joined_radius,
            left: a_index as i32,
            right: b_index as u32,
            _padding: [0; 2],
        }
    }
    pub(self) fn reflect_child_indices(&mut self, last_index: usize) {
        // Only if we are a branch
        if self.left >= 0 {
            self.left = (last_index as i32) - self.left;
            self.right = (last_index as u32) - self.right;
        }
    }
    pub(self) fn placeholder() -> Self {
        Self {
            pos: Vector3::zero(),
            radius: 0.0,
            left: 0,
            right: 0,
            _padding: [0; 2],
        }
    }
}
