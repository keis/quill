use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_mod_stylebuilder::StyleTuple;

use crate::{
    cx::Cx,
    effects::{self, AppendEffect, EffectTuple, EntityEffect},
    node_span::NodeSpan,
    style::ApplyStylesEffect,
    view::View,
    view_tuple::ViewTuple,
};

/// A view which generates an entity bundle.
#[derive(Default)]
pub struct Element<B: Bundle + Default = NodeBundle, C: ViewTuple = (), E: EffectTuple = ()> {
    /// Debug name for this element.
    debug_name: String,

    /// The visible UI node for this element.
    display: Option<Entity>,

    /// Children of this element.
    children: C,

    /// List of effects to be added to the element.
    effects: E,

    marker: PhantomData<B>,
}

impl<B: Bundle + Default> Element<B, (), ()> {
    /// Construct a new `Element`.
    pub fn new() -> Self {
        Self {
            debug_name: String::new(),
            display: None,
            children: (),
            effects: (),
            marker: PhantomData,
        }
    }

    /// Construct a new `Element` with a given entity id.
    pub fn for_entity(node: Entity) -> Self {
        Self {
            debug_name: String::new(),
            display: Some(node),
            children: (),
            effects: (),
            marker: PhantomData,
        }
    }
}

impl<B: Bundle + Default, C: ViewTuple, E: EffectTuple> Element<B, C, E> {
    /// Set the debug name for this element.
    pub fn named(mut self, name: &str) -> Self {
        self.debug_name = name.to_string();
        self
    }

    /// Set the child entities for this element.
    pub fn children<C2: ViewTuple>(self, children: C2) -> Element<B, C2, E> {
        Element {
            children,
            debug_name: self.debug_name,
            display: self.display,
            effects: self.effects,
            marker: PhantomData,
        }
    }

    /// Add an effect to this element.
    pub fn add_effect<E1: EntityEffect>(
        self,
        effect: E1,
    ) -> Element<B, C, <E as AppendEffect<E1>>::Result>
    where
        E: AppendEffect<E1>,
    {
        Element {
            children: self.children,
            debug_name: self.debug_name,
            display: self.display,
            effects: self.effects.append_effect(effect),
            marker: PhantomData,
        }
    }

    /// Apply a set of styles to the element
    pub fn style<S: StyleTuple + 'static>(
        self,
        styles: S,
    ) -> Element<B, C, <E as AppendEffect<ApplyStylesEffect<S>>>::Result>
    where
        E: AppendEffect<ApplyStylesEffect<S>>,
    {
        self.add_effect(ApplyStylesEffect { styles })
    }

    // pub fn insert_computed_ref<
    //     T: Component,
    //     F1: Send + Sync + 'static + FnMut() -> T,
    //     F2: Send + Sync + 'static + FnMut(&mut Re, &mut T),
    // >(
    //     mut self,
    //     init: F1,
    //     update: F2,
    // ) -> Self {
    //     self.producers.push(Arc::new(Mutex::new(BundleComputedRef {
    //         target: None,
    //         init,
    //         update,
    //         tracker: None,
    //         marker: PhantomData,
    //     })));
    //     self
    // }
}

// impl<B: Bundle + Default> EffectTarget for Element<B> {
// }

impl<B: Bundle + Default, C: ViewTuple, E: EffectTuple + 'static> View for Element<B, C, E> {
    type State = (Entity, C::State, E::State);

    fn nodes(&self, state: &Self::State) -> NodeSpan {
        NodeSpan::Node(state.0)
    }

    fn build(&self, cx: &mut Cx) -> Self::State {
        let owner = cx.owner;
        if self.debug_name.is_empty() {
            cx.world_mut()
                .entity_mut(owner)
                .insert(Name::new("Element"));
        } else {
            cx.world_mut()
                .entity_mut(owner)
                .insert(Name::new(format!("Element::{}", self.debug_name)));
        }

        // Build display entity if it doesn't already exist.
        let display = match self.display {
            Some(display) => {
                cx.world_mut()
                    .entity_mut(display)
                    .insert((B::default(), Name::new(self.debug_name.clone())));
                display
            }
            None => cx
                .world_mut()
                .spawn((B::default(), Name::new(self.debug_name.clone())))
                .id(),
        };

        // Run attached effects.
        let eff_state = effects::EffectTuple::apply(&self.effects, cx, display);

        // Build child nodes.
        let children = self.children.build_spans(cx);
        let nodes = self.children.span_nodes(&children);
        cx.world_mut()
            .entity_mut(display)
            .replace_children(&nodes.to_vec());
        (display, children, eff_state)
    }

    fn rebuild(&self, cx: &mut crate::cx::Cx, state: &mut Self::State) -> bool {
        effects::EffectTuple::reapply(&self.effects, cx, state.0, &mut state.2);
        if self.children.rebuild_spans(cx, &mut state.1) {
            let nodes = self.children.span_nodes(&state.1);
            cx.world_mut()
                .entity_mut(state.0)
                .replace_children(&nodes.to_vec());
        }
        false
    }

    fn raze(&self, world: &mut World, state: &mut Self::State) {
        // assert!(state.is_some());
        // self.raze_children(world);

        // Delete the display node.
        world.entity_mut(state.0).remove_parent();
        world.entity_mut(state.0).despawn();
        self.children.raze_spans(world, &mut state.1);
        // *state = None;

        // Delete all reactions and despawn the view entity.
        // world.despawn_owned_recursive(view_entity);
    }

    // fn children_changed(&mut self, _view_entity: Entity, world: &mut World) -> bool {
    //     self.attach_children(world);
    //     true
    // }
}

// impl<B: Bundle + Default, C: ViewTuple> IntoView for Element<B, C> {
//     fn into_view(self) -> Arc<Mutex<dyn AnyViewState>> {
//         Arc::new(Mutex::new(ViewState {
//             view: self,
//             state: None,
//         }))
//     }
// }
