use crate::ecs::EntityComponentSystem;
use crate::entity::EntityId;
use crate::input::Input;
use crate::graphics::Graphics;
use crate::render::RenderSystem;
use crate::control::ControlSystem;
use std::collections::HashSet;

pub trait System {
    fn run(&mut self, ecs: &mut EntityComponentSystem, entities: &HashSet<EntityId>, graphics: &mut Graphics, input: &Input, delta_time: f32);
    fn is_system_entity(&self, entity: &EntityId, ecs: &EntityComponentSystem) -> bool;
}

pub struct SystemManager {
    systems: Vec<(Box<dyn System>, HashSet<EntityId>)>,
}

impl SystemManager {
    pub fn new() -> Self {
        let mut systems : Vec<(Box<dyn System>, HashSet<EntityId>)> = Vec::new();

        // Systems are executed in order
        systems.push((Box::new(ControlSystem{}), HashSet::new()));
        systems.push((Box::new(RenderSystem{}), HashSet::new()));

        Self {
            systems
        }
    }

    pub fn run(&mut self, ecs: &mut EntityComponentSystem, graphics: &mut Graphics, input: &Input, delta_time: f32) {
        self.remove_entities_from_systems(&ecs.destroy_entities(), ecs);
        self.add_entities_to_systems(&ecs.create_entities(), ecs);

        for (system, entities) in &mut self.systems {
            system.run(ecs, &entities, graphics, input, delta_time);
        } 
    }

    fn add_entities_to_systems(&mut self, entities: &Vec<EntityId>, ecs: &EntityComponentSystem) {
        for entity in entities {
            for index in self.compatible_systems_indexes(&entity, ecs) {
                self.systems[index].1.insert(*entity);
            }
        }
    }

    fn remove_entities_from_systems(&mut self, entities: &Vec<EntityId>, ecs: &EntityComponentSystem) {
        for entity in entities {
            for index in self.compatible_systems_indexes(&entity, ecs) {
                self.systems[index].1.remove(entity);
            }
        }
    }

    fn compatible_systems_indexes(&self, entity: &EntityId, ecs: &EntityComponentSystem) -> Vec<usize> {
        self.systems.iter()
            .enumerate()
            .filter_map(|(index, system_entities)| if system_entities.0.is_system_entity(&entity, ecs) { Some(index) } else { None })
            .collect()
    }
}