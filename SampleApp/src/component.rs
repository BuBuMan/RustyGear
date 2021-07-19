use crate::entity::EntityId;

pub struct ArrayEntry<T> {
    pub value: T,
    generation: u64,
}

pub struct ComponentSet<T> {
    pub entries: Vec<Option<ArrayEntry<T>>>,
}

impl<T> ComponentSet<T> {
    pub fn new(max_size: usize) -> Self {
        let mut entries = Vec::new();
        entries.resize_with(max_size, Default::default);
        ComponentSet {
            entries
        }
    }

    // Set value for some index. May overwrite past generation.
    pub fn set(&mut self, gen_index: &EntityId, value: T) {
        debug_assert!(gen_index.index < self.entries.len());

        let new_entry = ArrayEntry {
            value: value,
            generation: gen_index.generation,
        };
        
        self.entries[gen_index.index] = Some(new_entry);
    }

    // Gets a constant value for some generational index. The generation must match.
    pub fn get(&self, gen_index: &EntityId) -> Option<&T> {
        debug_assert!(gen_index.index < self.entries.len());

        match &self.entries[gen_index.index] {
            None => None,
            Some(entry) => if entry.generation != gen_index.generation { None } else { Some(&entry.value) }
        }
    }

    // Gets a mutable value for some generational index. The generation must match.
    pub fn get_mut(&mut self, gen_index: &EntityId) -> Option<&mut T> {
        debug_assert!(gen_index.index < self.entries.len());

        match &mut self.entries[gen_index.index] {
            None => None,
            Some(entry) => if entry.generation !=  gen_index.generation { None } else { Some(&mut entry.value) }
        }
    }
}