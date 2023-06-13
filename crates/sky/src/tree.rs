use std::{
    array,
    f32::consts::{FRAC_1_SQRT_2, PI},
    iter::Flatten,
    mem,
    sync::Arc,
};

use ambient_gizmos::{Cuboid, Gizmo, GizmoPrimitive, DEFAULT_RADIUS};
use ambient_std::{color::Color, shapes::Ray};
use bytemuck::{Pod, Zeroable};
use derive_more::Deref;
use glam::{vec3, Vec3};
use noise::{NoiseFn, Perlin};
use ordered_float::NotNan;

/// Describes how to generate the terrain
pub trait Generator: Send + Sync + 'static {
    fn get(&self, point: Vec3) -> f32;
}

impl<F: NoiseFn<[f64; 3]> + Send + Sync + 'static> Generator for F {
    fn get(&self, p: Vec3) -> f32 {
        self.get([p.x as _, p.y as _, p.z as _]) as f32
    }
}

// 0 is a sentinel
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Pod, Zeroable, Deref)]
#[repr(transparent)]
pub struct NodeIndex(u32);

#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Node {
    density: f32,
    origin: Vec3,
    half_size: f32,
    children: [NodeIndex; 8],
    pad: [f32; 3],
}

#[derive(Clone)]
pub struct OctreeInfo {
    pub origin: Vec3,
    pub max_depth: u32,
    pub half_size: f32,
    pub generator: Arc<dyn Generator>,
    pub scale: f32,
}

impl OctreeInfo {
    pub fn build(self) -> Octree {
        Octree {
            nodes: vec![Node {
                density: 0.,
                origin: self.origin,
                half_size: self.half_size,
                ..Default::default()
            }],
            max_depth: self.max_depth,
            free: NodeIndex::default(),
            count: 1,
            generator: self.generator,
            scale: self.scale,
        }
    }
}

impl Default for OctreeInfo {
    fn default() -> Self {
        Self {
            max_depth: 10,
            half_size: 100.,
            generator: Arc::new(Perlin::new()),
            origin: Vec3::ZERO,
            scale: 0.1,
        }
    }
}

#[derive(Clone)]
/// Tree containing clouds in an octree
/// The root node is 0.
/// An "null" node reference is also 0 (can't use Option due to Pod).
/// Since the tree is acyclic the root is never referenced.
/// The root can also never be removed.
///
/// An internal node's density is always the maximum of any child or subchild.
pub struct Octree {
    nodes: Vec<Node>,
    max_depth: u32,
    free: NodeIndex,
    count: u32,
    generator: Arc<dyn Generator>,
    scale: f32,
}

impl Octree {
    // TODO make a free chain of children
    pub fn remove(&mut self, idx: NodeIndex) -> Node {
        let mut node = mem::replace(&mut self.nodes[*idx as usize], Node::free(self.free));
        self.free = idx;

        if !node.is_leaf() {
            for c in mem::take(&mut node.children) {
                assert_ne!(c, NodeIndex::root());
                self.remove(c);
            }
        }

        self.count -= 1;
        node
    }

    pub fn merge(&mut self, index: NodeIndex) -> u32 {
        let node = self.node_mut(index);
        if node.is_leaf() {
            0
        } else {
            let children = mem::take(&mut node.children);
            for c in children {
                self.remove(c);
            }
            1
        }
    }

    /// Split a node
    /// Does nothing for an internal node
    pub fn split(&mut self, index: NodeIndex) -> ([NodeIndex; 8], f32, u32) {
        let node = self.node(index);
        if !node.is_leaf() {
            (node.children, node.density, 0)
        } else {
            let size = node.half_size;
            let new_size = size / 2.;

            let mut children = [NodeIndex::default(); 8];
            let mut max_d = node.density;
            let leftmost = node.origin - Vec3::splat(new_size);

            for (i, c) in children.iter_mut().enumerate() {
                let off = vec3(
                    (i & 1 != 0) as u32 as f32 * size,
                    (i & 2 != 0) as u32 as f32 * size,
                    (i & 4 != 0) as u32 as f32 * size,
                );
                let pos = leftmost + off;

                let density = self.generator.get(pos * self.scale);
                max_d = max_d.max(density);
                let node = Node {
                    density,
                    origin: pos,
                    half_size: new_size,
                    ..Default::default()
                };

                *c = self.insert(node);
            }

            let node = self.node_mut(index);
            node.children = children;
            node.density = max_d;
            (children, max_d, 1)
        }
    }

    /// Update topotracingy by splitting or merging nodes
    /// `desired_size` desired radial size of the voxel
    pub fn update_topo(
        &mut self,
        index: NodeIndex,
        depth: u32,
        desired_size: f32,
        fov: f32,
        pos: Vec3,
    ) -> (f32, u32) {
        let max_depth = self.max_depth;
        let node = self.node_mut(index);

        let dist = pos.distance(node.origin) - node.half_size;
        let cur_size = node.half_size * FRAC_1_SQRT_2 / (PI * (dist) * fov);
        let mut d = node.density;

        let (children, u) = if (cur_size < 0. || cur_size > desired_size) && depth < max_depth {
            let (children, _, u) = self.split(index);
            (Some(children), u)
        }
        // Merge if node is less than half the desired size to give some leeway
        else if 4. * cur_size < desired_size {
            let d = node.density;
            let u = self.merge(index);
            return (d, u);
        } else {
            (node.children(), 0)
        };

        // We know that no child will update a parent
        let u = children.iter().flatten().fold(u, |u, &c| {
            assert_ne!(c, NodeIndex::root());
            let (new_d, new_n) = self.update_topo(c, depth + 1, desired_size, fov, pos);
            d = d.max(new_d);
            u + new_n
        });

        // Update density reading
        self.node_mut(index).density = d;
        (d, u)
    }

    pub fn node(&self, index: NodeIndex) -> &Node {
        &self.nodes[index.0 as usize]
    }

    pub fn node_mut(&mut self, index: NodeIndex) -> &mut Node {
        &mut self.nodes[index.0 as usize]
    }

    pub fn len(&self) -> u32 {
        self.count
    }

    pub fn query<F>(&self, accept: F) -> TreeQuery<F>
    where
        F: Fn(NodeIndex, &Node) -> bool,
    {
        TreeQuery::new(self, accept)
    }

    pub fn raycast(&self, ray: &Ray, d_threshold: f32) -> Option<RayIntersect> {
        let distance = self.raycast_inner(NodeIndex::root(), ray, d_threshold)?;
        Some(RayIntersect {
            point: ray.origin + ray.dir * distance,
            distance,
        })
    }

    fn raycast_inner(&self, index: NodeIndex, ray: &Ray, d_threshold: f32) -> Option<f32> {
        let node = self.node(index);
        let int = node.ray_intersect(ray, d_threshold)?;
        // If leaf, only check with self and return it
        if node.is_leaf() {
            Some(int)
        }
        // Internal node:
        // Density is the max(children); if self passes, some child density
        // should pass as well.
        //
        // iotw: if self succeeds, so do its parents
        else {
            node.children()
                .unwrap()
                .iter()
                .flat_map(|c| {
                    Some((
                        c,
                        self.raycast_inner(*c, ray, d_threshold)
                            .and_then(|v| NotNan::new(v).ok())?,
                    ))
                })
                .min_by_key(|(_, v)| *v)
                .map(|(_, v)| *v)
        }
    }

    pub fn insert(&mut self, node: Node) -> NodeIndex {
        self.count += 1;
        if self.free.is_valid() {
            // Init old node
            let idx = self.free;
            self.free = self.nodes[*idx as usize].next_free();
            self.nodes[*idx as usize] = node;
            idx
        } else {
            let idx = NodeIndex(self.nodes.len().try_into().unwrap());
            self.nodes.push(node);
            idx
        }
    }

    /// Get a reference to the octree's nodes.
    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        self.nodes.as_ref()
    }

    fn _root(&self) -> &Node {
        self.node(NodeIndex::root())
    }

    pub fn gizmos(&self, d_threshold: f32) -> Gizmos {
        Gizmos {
            query: self.query(|_, _| true),
            d_threshold,
        }
    }
}

pub struct TreeQuery<'a, F> {
    tree: &'a Octree,
    stack: Vec<NodeIndex>,
    accept: F,
}

impl<'a, F> TreeQuery<'a, F> {
    pub fn new(tree: &'a Octree, accept: F) -> Self {
        Self {
            tree,
            stack: vec![NodeIndex::root()],
            accept,
        }
    }
}

impl<'a, F> Iterator for TreeQuery<'a, F>
where
    F: Fn(NodeIndex, &Node) -> bool,
{
    type Item = (NodeIndex, &'a Node);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let index = self.stack.pop()?;

            let node = self.tree.node(index);

            if (self.accept)(index, node) {
                // Add children
                for c in node.children().iter().flatten() {
                    self.stack.push(*c)
                }

                return Some((index, node));
            }
        }
    }
}

#[doc(hidden)]
pub struct Gizmos<'a> {
    query: TreeQuery<'a, fn(NodeIndex, &Node) -> bool>,
    d_threshold: f32,
}

impl<'a> Gizmo for Gizmos<'a> {
    type Items = Flatten<Self>;

    fn into_gizmo_primitives(self) -> Self::Items {
        self.flatten()
    }
}

impl<'a> Iterator for Gizmos<'a> {
    type Item = std::array::IntoIter<GizmoPrimitive, 6>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (_, node) = self.query.next()?;
            if node.is_leaf() && node.density > self.d_threshold {
                return Some(
                    Cuboid::new(
                        node.origin,
                        Vec3::splat(node.half_size),
                        Vec3::ONE * (1. - node.density),
                        node.density,
                    )
                    .into_iter(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::vec3;

    use super::*;
    #[test]
    fn cloud_tree() {
        let mut tree = OctreeInfo::default().build();

        tree.split(NodeIndex::root());
        assert_eq!(tree.len(), 9);

        tree.merge(NodeIndex::root());
        tree.split(NodeIndex::root());
        assert_eq!(tree.nodes.len(), 9);
    }

    #[test]
    fn cloud_topo() {
        let mut tree = OctreeInfo::default().build();
        let center = vec3(12.0, 0., 0.0);

        tree.update_topo(NodeIndex::root(), 0, 0.1, 1.0, center);

        tracing::info!("Tree len: {}", tree.len());
    }
}

impl Node {
    fn free(next: NodeIndex) -> Node {
        Self {
            children: [
                next,
                NodeIndex::root(),
                NodeIndex::root(),
                NodeIndex::root(),
                NodeIndex::root(),
                NodeIndex::root(),
                NodeIndex::root(),
                NodeIndex::root(),
            ],
            ..Self::default()
        }
    }

    fn is_leaf(&self) -> bool {
        !self.children[0].is_valid()
    }

    fn next_free(&self) -> NodeIndex {
        self.children[0]
    }

    fn ray_intersect(&self, ray: &Ray, d_threshold: f32) -> Option<f32> {
        let dir = ray.dir;
        let origin = ray.origin - self.origin;
        let inv_dir = dir.recip();

        let t1 = (-self.half_size - origin) * inv_dir;
        let t2 = (self.half_size - origin) * inv_dir;

        let tmin = t1.min(t2);
        let tmax = t1.max(t2);

        let tmin = tmin.max_element();
        let tmax = tmax.min_element();

        if self.density > d_threshold && tmax > 0. && tmax >= tmin {
            Some(tmin)
        } else {
            None
        }
    }

    #[inline]
    fn children(&self) -> Option<[NodeIndex; 8]> {
        if self.is_leaf() {
            None
        } else {
            Some(self.children)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct RayIntersect {
    pub point: Vec3,
    pub distance: f32,
}

impl IntoIterator for RayIntersect {
    type IntoIter = array::IntoIter<GizmoPrimitive, 1>;

    type Item = GizmoPrimitive;

    fn into_iter(self) -> Self::IntoIter {
        [GizmoPrimitive::sphere(self.point, DEFAULT_RADIUS)
            .with_color(Color::hsl(self.distance / 100., 1., 0.5).into())]
        .into_iter()
    }
}

impl NodeIndex {
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }

    pub fn root() -> NodeIndex {
        Self(0)
    }
}
