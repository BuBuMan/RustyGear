use crate::component::ComponentSet;
use crate::entity::*;
use crate::transform::Transform;
use crate::camera::Camera;
use crate::controller::Controller;
use crate::mesh::Mesh;
use crate::resources::Resources;

use anymap::AnyMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::cell::RefCell;

pub struct EntityComponentSystem {
    entity_allocator: EntityAllocator,

    components: AnyMap,
    cameras: HashSet<EntityId>,
    entities_to_create: VecDeque<String>,
    entities_to_destroy: VecDeque<EntityId>,
    resources: Resources,
}

impl EntityComponentSystem {
    pub fn new(max_entities: usize, resources: Resources) -> Self {
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
            entities_to_create: VecDeque::new(),
            entities_to_destroy: VecDeque::new(),
            resources,
        }
    }

    pub fn get_component_set<T: 'static>(&self) -> Option<&RefCell<ComponentSet<T>>> {
        self.components.get::<RefCell<ComponentSet<T>>>()
    }

    pub fn add_entity(&mut self, prefab: String) {
        self.entities_to_create.push_back(prefab)
    }

    pub fn remove_entity(&mut self, entity: EntityId) {
        self.entities_to_destroy.push_back(entity)
    }

    // Must be called by system manager only so it can add the new entities to their corresponding systems. TODO: Figure out a better way
    pub fn create_entities(&mut self) -> Vec<EntityId> {
        let mut new_entities = Vec::new();
        while !self.entities_to_create.is_empty() {
            let prefab = self.entities_to_create.pop_front().unwrap();
            new_entities.push(self.create_entity(&prefab));
        }

        new_entities
    }

    // Must be called by system manager only so it can add the new entities to their corresponding systems. TODO: Figure out a better way
    pub fn destroy_entities(&mut self) -> Vec<EntityId> {
        let mut destroyed_entities = Vec::new();
        while !self.entities_to_destroy.is_empty() {
            let entity = self.entities_to_destroy.pop_front().unwrap();
            self.destroy_entity(&entity);
            destroyed_entities.push(entity);
        } 

        destroyed_entities
    }

    pub fn has_component<T: 'static>(&self, entity: &EntityId) -> bool {
        match self.get_component_set::<T>() {
            Some(set) => !set.borrow().get(&entity).is_none(),
            None => false
        }
    }

    pub fn cameras(&self) -> &HashSet<EntityId> {
        &self.cameras
    }

    fn create_entity(&mut self, prefab: &String) -> EntityId {
        let json = &self.resources.prefabs[prefab];
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

                entity
            }
            _ => { 
                panic!("Failed to create an entity from json file. Expected a json object."); 
            }
        }
    }

    fn destroy_entity(&mut self, entity: &EntityId) {
        self.entity_allocator.deallocate(entity);
        self.clear_component::<Transform>(entity);
        self.clear_component::<Camera>(entity);
        self.clear_component::<Controller>(entity);
        self.clear_component::<Mesh>(entity);
    }

    fn add_component<T: 'static>(&self, entityId: &EntityId, component: T) {
        self.get_component_set::<T>().unwrap().borrow_mut().set(&entityId, Some(component))
    }

    fn clear_component<T: 'static>(&self, entityId: &EntityId) {
        self.get_component_set::<T>().unwrap().borrow_mut().set(&entityId, None)
    }
}