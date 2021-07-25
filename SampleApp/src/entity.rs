use std::collections::HashSet;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct EntityId {
    pub index: usize,
    pub generation: u64,
}

struct AllocatorEntry {
    generation: u64
}

pub struct EntityAllocator {
    entries: Vec<AllocatorEntry>,
    free: Vec<usize>,
    pub max_size: usize,
    pub active_entities: HashSet<EntityId>,
}

impl EntityAllocator {
    pub fn new(max_size: usize) -> Self {
        EntityAllocator {
            entries: Vec::new(),
            free: Vec::new(),
            max_size: max_size,
            active_entities: HashSet::new(),
        }
    }

    pub fn allocate(&mut self) -> EntityId {
        let gen_index = match self.free.is_empty() {
            true => self.add_new_entry(),
            false => self.reuse_entry()
        };

        if self.active_entities.insert(gen_index) == false {
            panic!("System error. Allocated an already existing generational index.");
        }

        gen_index
    }

    pub fn deallocate(&mut self, gen_index: &EntityId) {
        if self.active_entities.remove(gen_index) == false {
            panic!("System error. Attempt to deallocate a non existing generational index.")
        }

        self.free.push(gen_index.index);
    }

    fn add_new_entry(&mut self) -> EntityId {
        if self.entries.len() >= self.max_size {
            panic!("Out of memory. Exceeded the maximum allowed of indices {}", self.max_size);
        }
        
        let gen_index = EntityId {
            index: self.entries.len(),
            generation: 0,
        };
        
        self.entries.push(AllocatorEntry {
            generation: 0,
        });

        gen_index
    }

    fn reuse_entry(&mut self) -> EntityId {
        let free_index = self.free.pop().unwrap();
        self.entries[free_index].generation += 1;

        let gen_index = EntityId {
            index: free_index,
            generation: self.entries[free_index].generation,
        };

        gen_index
    }
}

#[cfg(test)]
mod tests {
    use super::EntityAllocator;
    use super::EntityId;

    #[test]
    fn consecutive_allocates_increments_index_with_generation_zero() {
        let mut allocator = EntityAllocator::new(5);
        let index_zero = allocator.allocate();
        let index_one = allocator.allocate();
        let index_two = allocator.allocate();
        let generation = 0;
        assert_eq!(index_zero.index, 0);
        assert_eq!(index_zero.generation, generation);
        assert_eq!(index_one.index, 1);
        assert_eq!(index_one.generation, generation);
        assert_eq!(index_two.index, 2);
        assert_eq!(index_two.generation, generation);
    }

    #[test]
    fn dealloc_after_alloc_succeeds() {
        let mut allocator = EntityAllocator::new(5);
        let gen_index = allocator.allocate();
        allocator.deallocate(&gen_index);
    }

    #[test]
    #[should_panic]
    fn dealloc_without_alloc_fails() {
        let mut allocator = EntityAllocator::new(5);
        
        allocator.deallocate(&EntityId {
            index: 0,
            generation: 0,
        });
    }

    #[test]
    fn alloc_dealloc_alloc_returns_index_zero_gen_one() {
        let mut allocator = EntityAllocator::new(5);
        let gen_index = allocator.allocate();
        allocator.deallocate(&gen_index);
        let gen_index = allocator.allocate();
        assert_eq!(gen_index.index, 0);
        assert_eq!(gen_index.generation, 1);
    }

    #[test]
    #[should_panic]
    fn alloc_more_than_max_size_panics() {
        let mut allocator = EntityAllocator::new(1);
        allocator.allocate();
        allocator.allocate();
    }
}