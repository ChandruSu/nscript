use std::collections::HashMap;

use crate::vm::Value;

#[derive(Debug)]
pub enum GCObject {
    None,
    Object(HashMap<Value, Value>),
    Closure(Vec<Value>),
}

struct GCNode {
    item: GCObject,
    prev: Option<usize>,
    next: usize,
    marked: bool,
}

impl GCNode {
    pub fn empty(prev: Option<usize>, next: usize) -> Self {
        Self {
            item: GCObject::None,
            next,
            prev,
            marked: true,
        }
    }
}

pub struct Heap {
    slots: Vec<GCNode>,
    tail: Option<usize>,
}

// TODO: create trait for Allocator interface
impl Heap {
    pub fn new(initial_size: usize) -> Self {
        Self {
            tail: None,
            slots: (0..initial_size)
                .into_iter()
                .map(|i| GCNode::empty(i.checked_sub(1), i + 1))
                .collect(),
        }
    }

    pub fn get(&self, i: usize) -> &GCObject {
        &self.slots[i].item
    }

    pub fn get_mut(&mut self, i: usize) -> &mut GCObject {
        &mut self.slots[i].item
    }

    pub fn dump(&self) {
        println!("Heap Dump (tail = {:?}):", self.tail);

        for (i, item) in self.slots.iter().enumerate() {
            println!("Node {} content {:?}", i, item.item)
        }
        println!()
    }

    pub fn alloc(&mut self, n: GCObject) -> usize {
        let loc = self.tail.map(|i| self.slots[i].next).unwrap_or(0);

        if loc >= self.slots.len() {
            self.slots.extend(
                (self.slots.len()..self.slots.len() * 2)
                    .into_iter()
                    .map(|i| GCNode::empty(i.checked_sub(1), i + 1)),
            );
            self.slots[loc].prev = self.tail;
        }

        self.slots[loc].item = n;
        self.tail = Some(loc);

        loc
    }

    pub fn free(&mut self, loc: usize) {
        self.slots[loc].item = GCObject::None;
        match self.tail {
            Some(tail) if tail == loc => self.tail = self.slots[tail].prev,
            Some(tail) => {
                let prev = self.slots[loc].prev;
                let next = self.slots[loc].next;
                let free = self.slots[tail].next;

                self.slots[loc].next = free;
                if free < self.slots.len() {
                    self.slots[free].prev = Some(loc);
                }

                self.slots[loc].prev = Some(tail);
                self.slots[tail].next = loc;

                if next < self.slots.len() {
                    self.slots[next].prev = prev;
                }

                if let Some(prev) = prev {
                    self.slots[prev].next = next;
                }
            }
            None => unreachable!(),
        }
    }
}
