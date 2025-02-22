use bevy::prelude::{Bundle, Component, Entity};

use crate::{effects::EntityEffect, Cx};

/// Inserts a bundle into the target. If the deps change, then the bundle will be recomputed
/// and reinserted.
#[derive(Clone)]
pub struct InsertBundleEffect<B: Bundle + Clone, F: Fn(D) -> B + Clone, D: PartialEq + Clone> {
    pub factory: F,
    pub deps: D,
}

impl<
        B: Bundle + Clone,
        F: Fn(D) -> B + Send + Sync + Clone,
        D: PartialEq + Clone + Send + Sync,
    > EntityEffect for InsertBundleEffect<B, F, D>
{
    type State = D;
    fn apply(&self, cx: &mut Cx, target: Entity) -> Self::State {
        let mut target = cx.world_mut().entity_mut(target);
        target.insert((self.factory)(self.deps.clone()));
        self.deps.clone()
    }

    fn reapply(&self, cx: &mut Cx, target: Entity, state: &mut Self::State) {
        if *state != self.deps {
            *state = self.apply(cx, target);
        }
    }
}

/// Conditionally inserts a bundle into the target. If the condition is true, then the bundle
/// will be inserted. If the condition later becomes false, the component will be removed.
#[derive(Clone)]
pub struct ConditionalInsertComponentEffect<B: Bundle, F: Fn() -> B + Clone> {
    pub factory: F,
    pub condition: bool,
}

impl<C: Component + Clone, F: Fn() -> C + Send + Sync + Clone> EntityEffect
    for ConditionalInsertComponentEffect<C, F>
{
    type State = bool;
    fn apply(&self, cx: &mut Cx, target: Entity) -> Self::State {
        if self.condition {
            let mut target = cx.world_mut().entity_mut(target);
            target.insert((self.factory)());
        }
        self.condition
    }

    fn reapply(&self, cx: &mut Cx, target: Entity, state: &mut Self::State) {
        if self.condition != *state {
            *state = self.condition;
            if self.condition {
                self.apply(cx, target);
            } else {
                let mut target = cx.world_mut().entity_mut(target);
                target.remove::<C>();
            }
        }
    }
}

/// Inserts a bundle into the target once and never updates it.
#[derive(Clone)]
pub struct StaticInsertBundleEffect<B: Bundle + Clone> {
    pub bundle: B,
}

impl<B: Bundle + Clone> EntityEffect for StaticInsertBundleEffect<B> {
    type State = ();
    fn apply(&self, cx: &mut Cx, target: Entity) -> Self::State {
        let mut target = cx.world_mut().entity_mut(target);
        target.insert(self.bundle.clone());
    }

    fn reapply(&self, _cx: &mut Cx, _target: Entity, _state: &mut Self::State) {}
}
