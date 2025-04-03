use std::collections::HashMap;

use crate::vm::Value;

#[derive(Debug)]
pub enum HeapNode {
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
    Free {
        next: usize,
    },
}

impl HeapNode {
    pub fn free(next: usize) -> Self {
        Self::Free { next }
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

    pub fn children(&self) -> Vec<usize> {
        let get_ptr = |v: &Value| match v {
            Value::Func(_, p) => Some(*p as usize),
            Value::Object(p) => Some(*p),
            Value::Array(p) => Some(*p),
            _ => None,
        };

        match self {
            HeapNode::Closure { mark: _, vals } => vals.iter().filter_map(get_ptr).collect(),
            HeapNode::Object { mark: _, map } => map.values().filter_map(get_ptr).collect(),
            HeapNode::Array { mark: _, vec } => vec.iter().filter_map(get_ptr).collect(),
            _ => unreachable!(),
        }
    }
}

pub trait Alloc<P> {
    fn access(&self, ptr: P) -> &HeapNode;

    fn access_mut(&mut self, ptr: P) -> &mut HeapNode;

    fn allocate(&mut self, value: HeapNode) -> P;

    fn deallocate(&mut self, ptr: P);
}

pub struct Heap {
    nodes: Vec<HeapNode>,
    occupied: usize,
    head: usize,
    gc_threshold: usize,
}

impl Heap {
    pub fn new(capacity: usize) -> Self {
        Self {
            head: 0,
            occupied: 0,
            nodes: (0..capacity).map(|i| HeapNode::free(i + 1)).collect(),
            gc_threshold: capacity / 2,
        }
    }

    pub fn mark(&mut self, ptr: usize) {
        if self.nodes[ptr].marked() {
            return;
        }

        self.nodes[ptr].mark();

        for child in self.nodes[ptr].children() {
            self.mark(child);
        }
    }

    pub fn sweep(&mut self) {
        for p in 0..self.nodes.len() {
            if self.nodes[p].marked() {
                self.nodes[p].unmark();
            } else {
                self.deallocate(p);
            }
        }

        self.gc_threshold = self.occupied * 2;
    }

    pub fn should_collect(&self) -> bool {
        self.occupied >= self.gc_threshold
    }
}

impl Alloc<usize> for Heap {
    fn allocate(&mut self, value: HeapNode) -> usize {
        if let HeapNode::Free { next: _ } = value {
            unreachable!("Cannot allocate a free node");
        }

        let size = self.nodes.capacity();
        if self.head >= size {
            self.nodes
                .extend((size..2 * size).map(|i| HeapNode::free(i + 1)));
        }

        let ptr = self.head;
        self.head = match self.nodes[self.head] {
            HeapNode::Free { next } => next,
            _ => unreachable!("Head should always point to free node"),
        };

        self.nodes[ptr] = value;
        self.occupied += 1;
        ptr
    }

    fn deallocate(&mut self, ptr: usize) {
        match self.nodes[ptr] {
            HeapNode::Free { next: _ } => return,
            _ => {
                self.nodes[ptr] = HeapNode::free(self.head);
                self.head = ptr;
                self.occupied -= 1
            }
        }
    }

    fn access(&self, ptr: usize) -> &HeapNode {
        &self.nodes[ptr]
    }

    fn access_mut(&mut self, ptr: usize) -> &mut HeapNode {
        &mut self.nodes[ptr]
    }
}
