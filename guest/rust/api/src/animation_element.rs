use ambient_api_core::{
    core::{
        animation::{
            self,
            components::{play_clip_from_url, start_time},
        },
        app::components::name,
    },
    prelude::epoch_time,
};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};

/// An animation player
#[element_component]
pub fn AnimationPlayer(_hooks: &mut Hooks, root: Element) -> Element {
    Element::new()
        .with(animation::components::is_animation_player(), ())
        .children(vec![root])
        .with(name(), "Animation player".to_string())
}

/// Play an animation clip from an URL
#[element_component]
pub fn PlayClipFromUrl(
    _hooks: &mut Hooks,
    /// Url to clip
    url: String,
    /// Loop the animation
    looping: bool,
) -> Element {
    Element::new()
        .with(play_clip_from_url(), url.into())
        .with(name(), "Play clip from URL".to_string())
        .with(animation::components::looping(), looping)
        .init(start_time(), epoch_time())
}

/// Blend animation clips together
#[element_component(without_el)]
pub fn BlendNode(
    _hooks: &mut Hooks,
    /// Left animation node
    left: Element,
    /// Right animation node
    right: Element,
    /// Weight (0 means left, 1 means right, 0.5 means half left and half right)
    weight: f32,
) -> Element {
    if weight <= 0. {
        left
    } else if weight >= 1. {
        right
    } else {
        Element::new()
            .with(animation::components::blend(), weight)
            .with(name(), "Blend".to_string())
            .children(vec![left, right])
    }
}
impl BlendNode {
    /// Create a blend node and turn it into an Element
    pub fn el(left: Element, right: Element, weight: f32) -> Element {
        if weight <= 0. {
            left
        } else if weight >= 1. {
            right
        } else {
            Self {
                left,
                right,
                weight,
            }
            .el()
        }
    }
    /// Creates a tree of blend nodes where the weights are arbitrary, for example
    /// `BlendNode::normalize_multiblend(vec![("a", 1.), ("b", 20.), ("c", 3.)])` will create a tree
    /// where b is the strongest contribution
    pub fn normalize_multiblend(items: Vec<(Element, f32)>) -> Element {
        let total_weight = items.iter().map(|x| x.1).sum::<f32>();
        if total_weight <= 0. {
            return Element::new();
        }
        let mut items = items
            .into_iter()
            .map(|(a, w)| (a, w / total_weight))
            .collect::<Vec<_>>();
        items.retain(|x| x.1 > 0.);
        items.sort_by_key(|x| -ordered_float::OrderedFloat(x.1));
        for x in items.iter_mut() {
            x.1 = 1. - x.1;
        }
        Self::multiblend(items)
    }
    /// Creates a tree of blend nodes, where each weight is the blend between that element and the next,
    /// for example:
    /// `BlendNode::multiblend(vec![("a", 0.5), ("b", 0.2), ("c", 0.)])` will create a tree where
    /// b is first blended with c, using 20% of b and 80% of b
    /// The result if that is then blended with a, using 50% of the result and 50% of a
    pub fn multiblend(mut items: Vec<(Element, f32)>) -> Element {
        if items.len() == 0 {
            Element::new()
        } else if items.len() == 1 {
            items.pop().unwrap().0
        } else {
            let item = items.remove(0);
            Self::el(item.0, Self::multiblend(items), item.1)
        }
    }
}

/// Transition between multiple animations
#[element_component]
pub fn Transition(
    hooks: &mut Hooks,
    /// The animations that can be transitioned between
    animations: Vec<Element>,
    /// The index of the active animation
    active: usize,
    /// The speed that the transitions happen at
    speed: f32,
) -> Element {
    let weights = hooks.use_ref_with(|_| {
        animations
            .iter()
            .enumerate()
            .map(|(i, _)| if i == active { 1. } else { 0. })
            .collect::<Vec<_>>()
    });
    let mut weights = weights.lock();
    for (i, weight) in weights.iter_mut().enumerate() {
        let target = if i == active { 1. } else { 0. };
        *weight = *weight * (1. - speed) + target * speed;
    }
    BlendNode::normalize_multiblend(
        animations
            .into_iter()
            .zip(weights.iter().cloned())
            .collect(),
    )
}
