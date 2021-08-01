use crate::component::ComponentSet;
use crate::entity::*;
use crate::transform::Transform;
use crate::camera::Camera;
use crate::controller::Controller;
use crate::mesh::Mesh;

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
        components.insert(RefCell::new(ComponentSet::<Mesh>::new(max_entities)));

        Self {
            entity_allocator,
            components,
            cameras: HashSet::new(),
        }
    }

    pub fn get_component_set<T: 'static>(&self) -> Option<&RefCell<ComponentSet<T>>> {
        self.components.get::<RefCell<ComponentSet<T>>>()
    }

    pub fn create_entity(&mut self, json: &serde_json::Value) {
        match json {
            serde_json::Value::Object(object) => {
                let entity = self.entity_allocator.allocate();

                for key in object.keys() {
                    match key.as_ref() {
                        "Transform" => {
                            let component : Transform = serde_json::from_str(&object["Transform"].to_string()).unwrap();
                            self.add_component(&entity, component);
                        }
                        "Camera" => {
                            let component : Camera = serde_json::from_str(&object["Camera"].to_string()).unwrap();
                            self.add_component(&entity, component);
                        }
                        "Controller" => {
                            let component : Controller = serde_json::from_str(&object["Controller"].to_string()).unwrap();
                            self.add_component(&entity, component);
                        }
                        "Mesh" => {
                            let component : Mesh = serde_json::from_str(&object["Mesh"].to_string()).unwrap();
                            self.add_component(&entity, component);
                        }
                        _ => {}
                    };
                }

                if self.has_component::<Camera>(&entity) {
                    self.cameras.insert(entity);
                }
            }
            _ => { 
                panic!("Failed to create an entity from json file. Expected a json object."); 
            }
        }
    }

    pub fn active_entities(&self) -> &HashSet<EntityId> {
        &self.entity_allocator.active_entities
    }

    pub fn cameras(&self) -> &HashSet<EntityId> {
        &self.cameras
    }

    fn add_component<T: 'static>(&self, entityId: &EntityId, component: T) {
        self.get_component_set::<T>().unwrap().borrow_mut().set(&entityId, component)
    }

    fn has_component<T: 'static>(&self, entity: &EntityId) -> bool {
        match self.get_component_set::<T>() {
            Some(set) => !set.borrow().get(&entity).is_none(),
            None => false
        }
    }
}