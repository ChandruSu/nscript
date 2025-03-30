use std::collections::HashMap;

use crate::vm::Value;

#[derive(Debug)]
pub enum GCObject {
    Closure {
        mark: bool,
        vals: Vec<Value>,
    },
    Object {
        mark: bool,
        map: HashMap<Value, Value>,
    },
    Array {
        mark: bool,
        vec: Vec<Value>,
    },
    None {
        next: usize,
    },
}

impl GCObject {
    pub fn empty(next: usize) -> Self {
        Self::None { next }
    }

    pub fn object(map: HashMap<Value, Value>) -> Self {
        Self::Object { mark: false, map }
    }

    pub fn array(vec: Vec<Value>) -> Self {
        Self::Array { mark: false, vec }
    }

    pub fn closure(vals: Vec<Value>) -> Self {
        Self::Closure { mark: false, vals }
    }

    pub fn mark(&mut self) {
        match self {
            Self::Closure { mark, vals: _ } => *mark = true,
            Self::Object { mark, map: _ } => *mark = true,
            Self::Array { mark, vec: _ } => *mark = true,
            _ => unreachable!(),
        }
    }

    pub fn unmark(&mut self) {
        match self {
            Self::Closure { mark, vals: _ } => *mark = false,
            Self::Object { mark, map: _ } => *mark = false,
            Self::Array { mark, vec: _ } => *mark = false,
            _ => {}
        }
    }

    pub fn marked(&self) -> bool {
        match self {
            Self::Closure { mark, vals: _ } => *mark,
            Self::Object { mark, map: _ } => *mark,
            Self::Array { mark, vec: _ } => *mark,
            _ => true,
        }
    }
}

pub trait Alloc<P> {
    fn access(&self, ptr: P) -> &GCObject;

    fn access_mut(&mut self, ptr: P) -> &mut GCObject;

    fn alloc(&mut self, value: GCObject) -> P;

    fn free(&mut self, ptr: P);
}

pub struct Heap {
    slots: Vec<GCObject>,
    occupied: usize,
    head: usize,
    collection_threshold: usize,
}

impl Heap {
    pub fn new(capacity: usize) -> Self {
        Self {
            head: 0,
            occupied: 0,
            slots: (0..capacity).map(|i| GCObject::empty(i + 1)).collect(),
            collection_threshold: capacity / 2,
        }
    }

    pub fn dump(&self) {
        println!("Heap Dump {:p} head = {}", self, self.head);
        for (i, item) in self.slots.iter().enumerate() {
            println!("H({}) = {:?}", i, item);
        }
    }

    pub fn mark(&mut self, ptr: usize) {
        // Prevents infinite recursion from cyclic reference
        if self.slots[ptr].marked() {
            return;
        }

        self.slots[ptr].mark();

        let get_ptr = |v: &Value| match v {
            Value::Func(_, p) => Some(*p as usize),
            Value::Object(p) => Some(*p),
            Value::Array(p) => Some(*p),
            _ => None,
        };

        let child_nodes: Vec<usize> = match &self.slots[ptr] {
            GCObject::Closure { mark: _, vals } => vals.iter().filter_map(get_ptr).collect(),
            GCObject::Object { mark: _, map } => map.values().filter_map(get_ptr).collect(),
            GCObject::Array { mark: _, vec } => vec.iter().filter_map(get_ptr).collect(),
            _ => unreachable!(),
        };

        for &child in child_nodes.iter() {
            self.mark(child);
        }
    }

    pub fn sweep(&mut self) {
        for p in 0..self.slots.len() {
            if self.slots[p].marked() {
                self.slots[p].unmark();
            } else {
                self.free(p);
            }
        }

        self.collection_threshold = self.occupied * 2;
    }

    pub fn should_collect(&self) -> bool {
        self.occupied >= self.collection_threshold
    }
}

impl Alloc<usize> for Heap {
    fn alloc(&mut self, value: GCObject) -> usize {
        if let GCObject::None { next: _ } = value {
            unreachable!("Should allocate an empty slot!");
        }

        let size = self.slots.capacity();
        if self.head >= size {
            self.slots
                .extend((size..2 * size).map(|i| GCObject::empty(i + 1)));
        }

        let pos = self.head;
        self.head = match self.slots[self.head] {
            GCObject::None { next } => next,
            _ => unreachable!(),
        };

        self.slots[pos] = value;
        self.occupied += 1;
        pos
    }

    fn free(&mut self, ptr: usize) {
        match self.slots[ptr] {
            GCObject::None { next: _ } => return,
            _ => {
                self.slots[ptr] = GCObject::empty(self.head);
                self.head = ptr;
                self.occupied -= 1
            }
        }
    }

    fn access(&self, ptr: usize) -> &GCObject {
        &self.slots[ptr]
    }

    fn access_mut(&mut self, ptr: usize) -> &mut GCObject {
        &mut self.slots[ptr]
    }
}
