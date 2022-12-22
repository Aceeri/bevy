use crate::{
    self as bevy_ecs,
    archetype::{Archetype, Archetypes},
    bundle::Bundles,
    change_detection::Ticks,
    component::{Component, ComponentId, ComponentTicks, Components, Tick},
    entity::{Entities, Entity},
    event::{Events, ManualEventReader},
    prelude::{Local, Res},
    query::{
        Access, FilteredAccess, FilteredAccessSet, QueryState, ReadOnlyWorldQuery, WorldQuery,
    },
    storage::SparseSet,
    system::{CommandQueue, Commands, Query, Resource, SystemMeta, ToComponentId},
    world::{FromWorld, World},
};
use bevy_ecs_macros::SystemParam;
use bevy_ecs_macros::{all_tuples, impl_param_set};
use bevy_ptr::UnsafeCellDeref;
use bevy_utils::synccell::SyncCell;
use std::{
    borrow::Cow,
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct RemovedComponentReader<T>
where
    T: Component,
{
    reader: ManualEventReader<Entity>,
    marker: PhantomData<T>,
}

impl<T: Component> Default for RemovedComponentReader<T> {
    fn default() -> Self {
        Self {
            reader: Default::default(),
            marker: PhantomData,
        }
    }
}

impl<T: Component> Deref for RemovedComponentReader<T> {
    type Target = ManualEventReader<Entity>;
    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}

impl<T: Component> DerefMut for RemovedComponentReader<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}

#[derive(Default, Debug)]
pub struct RemovedComponentEvents {
    event_sets: SparseSet<ComponentId, Events<Entity>>,
}

impl RemovedComponentEvents {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self) {
        for (component_id, events) in self.event_sets.iter_mut() {
            events.update();
        }
    }

    pub fn get(&self, component_id: impl Into<ComponentId>) -> Option<&Events<Entity>> {
        self.event_sets
            .get(component_id.into())
    }

    pub fn expect(&self, component_id: impl Into<ComponentId>) -> &Events<Entity> {
        self.get(component_id)
            .expect("No removal events for component")
    }

    pub fn send(&mut self, component_id: impl Into<ComponentId>, entity: Entity) {
        self.event_sets
            .get_or_insert_with(component_id.into(), Default::default)
            .send(entity);
    }
}

#[derive(SystemParam)]
/// A [`SystemParam`] that grants access to the entities that had their `T` [`Component`] removed.
///
/// Note that this does not allow you to see which data existed before removal.
/// If you need this, you will need to track the component data value on your own,
/// using a regularly scheduled system that requests `Query<(Entity, &T), Changed<T>>`
/// and stores the data somewhere safe to later cross-reference.
///
/// If you are using `bevy_ecs` as a standalone crate,
/// note that the `RemovedComponents` list will not be automatically cleared for you,
/// and will need to be manually flushed using [`World::clear_trackers`]
///
/// For users of `bevy` and `bevy_app`, this is automatically done in `bevy_app::App::update`.
/// For the main world, [`World::clear_trackers`] is run after the main schedule is run and after
/// `SubApp`'s have run.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// # use bevy_ecs::component::Component;
/// # use bevy_ecs::system::IntoSystem;
/// # use bevy_ecs::system::RemovedComponents;
/// #
/// # #[derive(Component)]
/// # struct MyComponent;
///
/// fn react_on_removal(removed: RemovedComponents<MyComponent>) {
///     removed.iter().for_each(|removed_entity| println!("{:?}", removed_entity));
/// }
///
/// # bevy_ecs::system::assert_is_system(react_on_removal);
/// ```
pub struct RemovedComponents<'w, 's, T: Component> {
    component_id: ToComponentId<T>,
    reader: Local<'s, RemovedComponentReader<T>>,
    event_sets: &'w RemovedComponentEvents,
}

impl<'w, 's, T: Component> RemovedComponents<'w, 's, T> {
    pub fn iter(&mut self) -> Box<dyn Iterator<Item = Entity> + '_> {
        if let Some(events) = self.event_sets.get(self.component_id) {
            Box::new(self.reader.iter(events).cloned())
        } else {
            Box::new(std::iter::empty())
        }
    }
}
