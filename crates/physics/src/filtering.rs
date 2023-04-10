use ambient_ecs::{components, Debuggable, Networked, Store};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use thiserror::Error;

components!("physics", {
    @[Debuggable, Networked, Store]
    layer: Layer,
    @[Debuggable, Networked, Store]
    collision_filter: CollisionFilter,
});

/// Represents a distinct physics layer.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct Layer(u8);

impl Layer {
    /// Creates a new physics layer from a layer index.
    fn new(index: u8) -> Layer {
        Layer(index & 31)
    }
}

impl From<Layer> for usize {
    fn from(val: Layer) -> Self {
        val.0 as usize
    }
}

impl From<Layer> for LayerMask {
    fn from(val: Layer) -> Self {
        LayerMask { bits: 1 << val.0 }
    }
}

bitflags! {
    /// Represents a layer mask.
    ///
    /// Each flag represents a layer and is set if this layer is active in the given filter.
    #[derive(Serialize, Deserialize, Default)]
    pub struct LayerMask: u32 {
        const LAYER_1 = 1 << 0;
        const LAYER_2 = 1 << 1;
        const LAYER_3 = 1 << 2;
        const LAYER_4 = 1 << 3;
        const LAYER_5 = 1 << 4;
        const LAYER_6 = 1 << 5;
        const LAYER_7 = 1 << 6;
        const LAYER_8 = 1 << 7;
        const LAYER_9 = 1 << 8;
        const LAYER_10 = 1 << 9;
        const LAYER_11 = 1 << 10;
        const LAYER_12 = 1 << 11;
        const LAYER_13 = 1 << 12;
        const LAYER_14 = 1 << 13;
        const LAYER_15 = 1 << 14;
        const LAYER_16 = 1 << 15;
        const LAYER_17 = 1 << 16;
        const LAYER_18 = 1 << 17;
        const LAYER_19 = 1 << 18;
        const LAYER_20 = 1 << 19;
        const LAYER_21 = 1 << 20;
        const LAYER_22 = 1 << 21;
        const LAYER_23 = 1 << 22;
        const LAYER_24 = 1 << 23;
        const LAYER_25 = 1 << 24;
        const LAYER_26 = 1 << 25;
        const LAYER_27 = 1 << 26;
        const LAYER_28 = 1 << 27;
        const LAYER_29 = 1 << 28;
        const LAYER_30 = 1 << 29;
        const LAYER_31 = 1 << 30;
        const LAYER_32 = 1 << 31;
    }
}

/// Represents a collision filter on an object.
///
/// Must be exactly 128-bits, in order to align with physx's collision filter data.
#[derive(Serialize, Deserialize, Default, Debug, Copy, Clone, PartialEq)]
pub struct CollisionFilter {
    /// The layer this filter represents.
    pub layer: LayerMask,
    /// Layers this filter blocks.
    pub blocks: LayerMask,
    /// Layers this filter overlaps.
    pub overlaps: LayerMask,
    /// Reserved for future use. Will be used to look up expanded filter data if non-zero.
    pub _reserved: u32,
}

impl CollisionFilter {
    pub fn blocks(&self, filter: &CollisionFilter) -> bool {
        self.blocks.contains(filter.layer)
    }

    pub fn overlaps(&self, filter: &CollisionFilter) -> bool {
        self.overlaps.contains(filter.layer)
    }
}

#[derive(Error, Debug)]
pub enum LayerInfoError {
    #[error("No layers available")]
    LayersExhausted,
    #[error("Layer `{0:?}` does not exist")]
    UnknownLayer(Layer),
}

pub type LayerInfoResult<T> = std::result::Result<T, LayerInfoError>;

/// Represents the layer info for a given physics world.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct LayerInfo {
    layers: LayerMask,
    filters: [CollisionFilter; 32],
    metadata: [(String, String); 32],
}

impl LayerInfo {
    pub fn register_layer(&mut self, name: String, description: String) -> LayerInfoResult<Layer> {
        if self.layers.is_all() {
            Err(LayerInfoError::LayersExhausted)
        } else {
            let layer = Layer::new(self.layers.bits.trailing_ones() as u8);
            let layer_index = usize::from(layer);
            let layer_mask = LayerMask::from(layer);
            self.filters[layer_index] = CollisionFilter { layer: layer_mask, ..Default::default() };
            self.metadata[layer_index] = (name, description);
            self.layers |= layer_mask;
            Ok(layer)
        }
    }

    pub fn release_layer(&mut self, layer: Layer) -> LayerInfoResult<()> {
        let layer_mask = LayerMask::from(layer);
        if !self.layers.contains(layer_mask) {
            Err(LayerInfoError::UnknownLayer(layer))
        } else {
            let layer_index = usize::from(layer);
            self.layers -= layer_mask;
            self.filters[layer_index] = CollisionFilter::default();
            self.metadata[layer_index] = Default::default();
            Ok(())
        }
    }

    pub fn get_layer_name(&self, layer: Layer) -> LayerInfoResult<&String> {
        let layer_mask = LayerMask::from(layer);
        if !self.layers.contains(layer_mask) {
            Err(LayerInfoError::UnknownLayer(layer))
        } else {
            let layer_index = usize::from(layer);
            Ok(&self.metadata[layer_index].0)
        }
    }

    pub fn get_layer_description(&self, layer: Layer) -> LayerInfoResult<&String> {
        let layer_mask = LayerMask::from(layer);
        if !self.layers.contains(layer_mask) {
            Err(LayerInfoError::UnknownLayer(layer))
        } else {
            let layer_index = usize::from(layer);
            Ok(&self.metadata[layer_index].1)
        }
    }

    pub fn get_collision_filter(&self, layer: Layer) -> LayerInfoResult<&CollisionFilter> {
        let layer_mask = LayerMask::from(layer);
        if !self.layers.contains(layer_mask) {
            Err(LayerInfoError::UnknownLayer(layer))
        } else {
            let layer_index = usize::from(layer);
            Ok(&self.filters[layer_index])
        }
    }

    pub fn blocks(&mut self, layer_a: Layer, layer_b: Layer) -> LayerInfoResult<()> {
        let layer_mask_a = LayerMask::from(layer_a);
        let layer_mask_b = LayerMask::from(layer_b);
        if !self.layers.contains(layer_mask_a) {
            Err(LayerInfoError::UnknownLayer(layer_a))
        } else if !self.layers.contains(layer_mask_b) {
            Err(LayerInfoError::UnknownLayer(layer_b))
        } else {
            let layer_index_a = usize::from(layer_a);
            let layer_index_b = usize::from(layer_b);
            self.filters[layer_index_a].blocks |= layer_mask_b;
            self.filters[layer_index_b].blocks |= layer_mask_a;
            self.filters[layer_index_a].overlaps -= layer_mask_b;
            self.filters[layer_index_b].overlaps -= layer_mask_a;
            Ok(())
        }
    }

    pub fn overlaps(&mut self, layer_a: Layer, layer_b: Layer) -> LayerInfoResult<()> {
        let layer_mask_a = LayerMask::from(layer_a);
        let layer_mask_b = LayerMask::from(layer_b);
        if !self.layers.contains(layer_mask_a) {
            Err(LayerInfoError::UnknownLayer(layer_a))
        } else if !self.layers.contains(layer_mask_b) {
            Err(LayerInfoError::UnknownLayer(layer_b))
        } else {
            let layer_index_a = usize::from(layer_a);
            let layer_index_b = usize::from(layer_b);
            self.filters[layer_index_a].blocks -= layer_mask_b;
            self.filters[layer_index_b].blocks -= layer_mask_a;
            self.filters[layer_index_a].overlaps |= layer_mask_b;
            self.filters[layer_index_b].overlaps |= layer_mask_a;
            Ok(())
        }
    }

    pub fn ignores(&mut self, layer_a: Layer, layer_b: Layer) -> LayerInfoResult<()> {
        let layer_mask_a = LayerMask::from(layer_a);
        let layer_mask_b = LayerMask::from(layer_b);
        if !self.layers.contains(layer_mask_a) {
            Err(LayerInfoError::UnknownLayer(layer_a))
        } else if !self.layers.contains(layer_mask_b) {
            Err(LayerInfoError::UnknownLayer(layer_b))
        } else {
            let layer_index_a = usize::from(layer_a);
            let layer_index_b = usize::from(layer_b);
            self.filters[layer_index_a].blocks -= layer_mask_b;
            self.filters[layer_index_b].blocks -= layer_mask_a;
            self.filters[layer_index_a].overlaps -= layer_mask_b;
            self.filters[layer_index_b].overlaps -= layer_mask_a;
            Ok(())
        }
    }
}

#[test]
fn test_collision_filter_size() {
    assert!(core::mem::size_of::<CollisionFilter>() == 16);
}

#[test]
fn test_collision_filter_blocks() {
    let layer_1_filter = CollisionFilter { layer: LayerMask::LAYER_1, blocks: LayerMask::LAYER_2, ..Default::default() };
    let layer_2_filter = CollisionFilter { layer: LayerMask::LAYER_2, blocks: LayerMask::LAYER_1, ..Default::default() };
    // Ensure both filters block one another
    assert!(layer_1_filter.blocks(&layer_2_filter));
    assert!(layer_2_filter.blocks(&layer_1_filter));
    // Ensure both filters do not overlap one another
    assert!(!layer_1_filter.overlaps(&layer_2_filter));
    assert!(!layer_2_filter.overlaps(&layer_1_filter));
}

#[test]
fn test_collision_filter_overlaps() {
    let layer_1_filter = CollisionFilter { layer: LayerMask::LAYER_1, overlaps: LayerMask::LAYER_2, ..Default::default() };
    let layer_2_filter = CollisionFilter { layer: LayerMask::LAYER_2, overlaps: LayerMask::LAYER_1, ..Default::default() };
    // Ensure both filters do not block one another
    assert!(!layer_1_filter.blocks(&layer_2_filter));
    assert!(!layer_2_filter.blocks(&layer_1_filter));
    // Ensure both filters do not overlap one another
    assert!(layer_1_filter.overlaps(&layer_2_filter));
    assert!(layer_2_filter.overlaps(&layer_1_filter));
}

#[test]
fn test_layer_info_registration_and_release() {
    let mut layer_info = LayerInfo::default();
    let layer_name = "layer".to_string();
    let layer_description = "Layer description".to_string();
    // Register a layer with the given name and description
    let layer = layer_info.register_layer(layer_name.clone(), layer_description.clone()).unwrap();
    // Ensure we get the correct layer metadata
    assert_eq!(layer_info.get_layer_name(layer).unwrap(), &layer_name);
    assert_eq!(layer_info.get_layer_description(layer).unwrap(), &layer_description);
    assert_eq!(layer_info.get_collision_filter(layer).unwrap(), &CollisionFilter::default());
    // Ensure we can no longer get the layer metadata
    layer_info.release_layer(layer).unwrap();
    assert!(layer_info.get_layer_name(layer).is_err());
    assert!(layer_info.get_layer_description(layer).is_err());
    assert!(layer_info.get_collision_filter(layer).is_err());
}

#[test]
fn test_layer_info_exhaustion_and_recovery() {
    let mut layer_info = LayerInfo::default();
    // Register 32 layers
    let mut layers = vec![];
    for _ in 0..32 {
        layers.push(layer_info.register_layer(String::default(), String::default()).unwrap());
    }
    // Ensure that we can allocate no more layers, and then replace the current layer
    layers.iter().for_each(|layer| {
        assert!(layer_info.register_layer(String::default(), String::default()).is_err());
        layer_info.release_layer(*layer).unwrap();
        assert_eq!(*layer, layer_info.register_layer(String::default(), String::default()).unwrap());
    });
}

#[test]
fn test_layer_info_collision_filters() {
    let mut layer_info = LayerInfo::default();
    // Register layers
    let layer_1 = layer_info.register_layer(Default::default(), Default::default()).unwrap();
    let layer_2 = layer_info.register_layer(Default::default(), Default::default()).unwrap();
    // Ensure both layers initially ignore one another
    {
        let layer_collision_filter_1 = layer_info.get_collision_filter(layer_1).unwrap();
        let layer_collision_filter_2 = layer_info.get_collision_filter(layer_2).unwrap();
        assert!(!layer_collision_filter_1.blocks(layer_collision_filter_2));
        assert!(!layer_collision_filter_2.blocks(layer_collision_filter_1));
        assert!(!layer_collision_filter_1.overlaps(layer_collision_filter_2));
        assert!(!layer_collision_filter_2.overlaps(layer_collision_filter_1));
    }
    // Ensure both layers block one another, but do not overlap
    layer_info.blocks(layer_1, layer_2).unwrap();
    {
        let layer_collision_filter_1 = layer_info.get_collision_filter(layer_1).unwrap();
        let layer_collision_filter_2 = layer_info.get_collision_filter(layer_2).unwrap();
        assert!(layer_collision_filter_1.blocks(layer_collision_filter_2));
        assert!(layer_collision_filter_2.blocks(layer_collision_filter_1));
        assert!(!layer_collision_filter_1.overlaps(layer_collision_filter_2));
        assert!(!layer_collision_filter_2.overlaps(layer_collision_filter_1));
    }
    // Ensure both layers overlap one another, but do not block
    layer_info.overlaps(layer_1, layer_2).unwrap();
    {
        let layer_collision_filter_1 = layer_info.get_collision_filter(layer_1).unwrap();
        let layer_collision_filter_2 = layer_info.get_collision_filter(layer_2).unwrap();
        assert!(!layer_collision_filter_1.blocks(layer_collision_filter_2));
        assert!(!layer_collision_filter_2.blocks(layer_collision_filter_1));
        assert!(layer_collision_filter_1.overlaps(layer_collision_filter_2));
        assert!(layer_collision_filter_2.overlaps(layer_collision_filter_1));
    }
    // Ensure both layers ignore one another
    layer_info.ignores(layer_1, layer_2).unwrap();
    {
        let layer_collision_filter_1 = layer_info.get_collision_filter(layer_1).unwrap();
        let layer_collision_filter_2 = layer_info.get_collision_filter(layer_2).unwrap();
        assert!(!layer_collision_filter_1.blocks(layer_collision_filter_2));
        assert!(!layer_collision_filter_2.blocks(layer_collision_filter_1));
        assert!(!layer_collision_filter_1.overlaps(layer_collision_filter_2));
        assert!(!layer_collision_filter_2.overlaps(layer_collision_filter_1));
    }
}
