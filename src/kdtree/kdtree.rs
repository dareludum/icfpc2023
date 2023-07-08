use rayon::ThreadPoolBuilder;

use crate::kdtree::aabb::*;
use crate::kdtree::candidate::*;
use crate::kdtree::config::BuilderConfig;
use crate::kdtree::kdnode::{build_tree, KDTreeNode};
use crate::kdtree::ray::Ray;
use crate::kdtree::Vector2;

/// The KD-tree data structure.
#[derive(Clone, Debug)]
pub struct KDTree {
    tree: Vec<KDTreeNode>,
    space: AABB,
    depth: usize,
}

impl KDTree {
    /// This function is used to build a new KD-tree. You need to provide a
    /// `Vec` of shapes that implement `Bounded` trait.
    /// You also should give a configuration.
    /// Panic if the `shapes` is empty.
    pub fn build_config<S: Bounded>(shapes: &Vec<S>, config: &BuilderConfig) -> Self {
        assert!(!shapes.is_empty());
        let mut space = AABB::default();
        let mut candidates = Candidates::with_capacity(shapes.len() * 6);
        for (index, v) in shapes.iter().enumerate() {
            // Create items from values
            let bb = v.bound();
            candidates.extend(Candidate::gen_candidates(index, &bb));

            // Update space with the bounding box of the item
            space.merge(&bb);
        }

        // Sort candidates only once at the begining
        candidates.sort();

        let nb_shapes = shapes.len();

        // Build the tree
        let (depth, tree) = build_tree(config, &space, candidates, nb_shapes);

        KDTree { space, tree, depth }
    }

    /// This function is used to build a new KD-tree. You need to provide a
    /// `Vec` of shapes that implement `Bounded` trait.
    /// Take a default configuration.
    /// Panic if the `shapes` is empty.
    pub fn build<S: Bounded>(shapes: &Vec<S>) -> Self {
        Self::build_config(shapes, &BuilderConfig::default())
    }

    /// This function takes a ray and return a reduced list of shapes that
    /// can be intersected by the ray.
    pub fn intersect(&self, ray_origin: &Vector2, ray_direction: &Vector2) -> Vec<usize> {
        let ray = Ray::new(ray_origin, ray_direction);
        let mut result = vec![];
        let mut stack = vec![0];
        stack.reserve_exact(self.depth);
        while !stack.is_empty() {
            let node = &self.tree[stack.pop().unwrap()];
            match node {
                KDTreeNode::Leaf { shapes } => result.extend(shapes),
                KDTreeNode::Node {
                    l_child,
                    l_space,
                    r_child,
                    r_space,
                } => {
                    if ray.intersect(r_space) {
                        stack.push(*r_child)
                    }
                    if ray.intersect(l_space) {
                        stack.push(*l_child)
                    }
                }
            }
        }
        // Dedup duplicated shapes
        result.sort();
        result.dedup();
        result
    }
}

impl Bounded for KDTree {
    fn bound(&self) -> AABB {
        self.space.clone()
    }
}
