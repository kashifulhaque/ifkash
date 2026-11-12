//! A simple generational arena with deterministic (index-order) iteration.

use serde::{Deserialize, Serialize};

/// A handle to an arena slot. Stale handles (slot reused) fail to resolve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Id {
    pub index: u32,
    pub generation: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Slot<T> {
    generation: u32,
    value: Option<T>,
}

/// Slots are reused lowest-index-first, so behavior depends only on the call sequence.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self { slots: Vec::new() }
    }
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, value: T) -> Id {
        if let Some(index) = self.slots.iter().position(|s| s.value.is_none()) {
            let slot = &mut self.slots[index];
            slot.value = Some(value);
            return Id {
                index: index as u32,
                generation: slot.generation,
            };
        }
        self.slots.push(Slot {
            generation: 0,
            value: Some(value),
        });
        Id {
            index: (self.slots.len() - 1) as u32,
            generation: 0,
        }
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        let value = slot.value.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        Some(value)
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        let slot = self.slots.get(id.index as usize)?;
        if slot.generation == id.generation {
            slot.value.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation == id.generation {
            slot.value.as_mut()
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.slots.iter().filter(|s| s.value.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterates live entries in ascending slot-index order.
    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> {
        self.slots.iter().enumerate().filter_map(|(i, s)| {
            s.value.as_ref().map(|v| {
                (
                    Id {
                        index: i as u32,
                        generation: s.generation,
                    },
                    v,
                )
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_handles_fail_and_slots_are_reused() {
        let mut a = Arena::new();
        let x = a.insert(1);
        let y = a.insert(2);
        assert_eq!(a.remove(x), Some(1));
        assert_eq!(a.get(x), None);
        let z = a.insert(3);
        assert_eq!(z.index, x.index);
        assert_ne!(z.generation, x.generation);
        assert_eq!(a.iter().map(|(_, v)| *v).collect::<Vec<_>>(), vec![3, 2]);
        assert_eq!(a.get(y), Some(&2));
    }
}
