use crate::component::ComponentSet;
use crate::entity::*;
use crate::transform::Transform;
use crate::camera::Camera;
use crate::controller::Controller;

use anymap::AnyMap;
use std::collections::HashSet;
use std::cell::RefCell;

pub struct EntityComponentSystem {
    entity_allocator: EntityAllocator,

    components: AnyMap,
    cameras: HashSet<EntityId>,
}

impl EntityComponentSystem {
    pub fn new(max_entities: usize) -> Self {
        let entity_allocator = EntityAllocator::new(max_entities);

        let mut components = AnyMap::new();
        components.insert(RefCell::new(ComponentSet::<Transform>::new(max_entities)));
        components.insert(RefCell::new(ComponentSet::<Controller>::new(max_entities)));
        components.insert(RefCell::new(ComponentSet::<Camera>::new(max_entities)));

        Self {
            entity_allocator,
            components,
            cameras: HashSet::new(),
        }
    }

    pub fn get_component_set<T: 'static>(&self) -> Option<&RefCell<ComponentSet<T>>> {
        self.components.get::<RefCell<ComponentSet<T>>>()
    }

    pub fn create_entity(&mut self, transform: Option<Transform>, camera: Option<Camera>, controller: Option<Controller>) {
        let entity = self.entity_allocator.allocate();

        if !camera.is_none() {
            self.cameras.insert(entity);
        }

        self.add_component(&entity, transform);
        self.add_component(&entity, camera);
        self.add_component(&entity, controller);
    }

    pub fn active_entities(&self) -> &HashSet<EntityId> {
        &self.entity_allocator.active_entities
    }

    pub fn cameras(&self) -> &HashSet<EntityId> {
        &self.cameras
    }

    fn add_component<T: 'static>(&self, entityId: &EntityId, component: Option<T>) {
        match component {
            Some(value) => self.get_component_set::<T>().unwrap().borrow_mut().set(&entityId, value),
            _ => {}
        };
    }
}