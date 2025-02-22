use bevy::{prelude::Entity, ui};
use bevy_mod_stylebuilder::{StyleBuilder, StyleTuple};

use crate::{effects::EntityEffect, Cx};

/// Inserts a static, pre-constructed bundle into the target entity. No reactivity.
#[derive(Clone, PartialEq)]
pub struct ApplyStaticStylesEffect<S: StyleTuple + Clone> {
    pub styles: S,
}

impl<S: StyleTuple + Clone> EntityEffect for ApplyStaticStylesEffect<S> {
    type State = ();
    fn apply(&self, cx: &mut Cx, target: Entity) -> Self::State {
        let mut target = cx.world_mut().entity_mut(target);
        let mut node = ui::Node::default();
        if let Some(s) = target.get::<ui::Node>() {
            node.clone_from(s);
        }
        let mut sb = StyleBuilder::new(&mut target, node);
        self.styles.apply(&mut sb);
        sb.finish();
    }
}

/// Applies dynamic styles which are computed reactively. The `deps` field is used to determine
/// whether the styles need to be recomputed; if the deps have not changed since the previous
/// update cycle, then the styles are not recomputed.
#[derive(Clone)]
pub struct ApplyDynamicStylesEffect<F: Fn(D, &mut StyleBuilder) + Clone, D: PartialEq + Clone> {
    pub(crate) style_fn: F,
    pub(crate) deps: D,
}

impl<F: Fn(D, &mut StyleBuilder) + Send + Sync + Clone, D: PartialEq + Clone + Send + Sync>
    EntityEffect for ApplyDynamicStylesEffect<F, D>
{
    type State = D;
    fn apply(&self, cx: &mut Cx, target: Entity) -> Self::State {
        let mut target = cx.world_mut().entity_mut(target);
        let mut node = ui::Node::default();
        if let Some(s) = target.get::<ui::Node>() {
            node.clone_from(s);
        }
        let mut sb = StyleBuilder::new(&mut target, node);
        (self.style_fn)(self.deps.clone(), &mut sb);
        sb.finish();
        self.deps.clone()
    }

    fn reapply(&self, cx: &mut Cx, target: Entity, state: &mut Self::State) {
        if *state != self.deps {
            *state = self.apply(cx, target);
        }
    }
}
