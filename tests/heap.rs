use ns::vm::heap::{Alloc, GCObject, Heap};
use ns::vm::Value;

#[test]
pub fn test_heap() {
    let mut h = Heap::new(4);
    h.alloc(GCObject::closure(vec![Value::Int(10)]));
    h.alloc(GCObject::closure(vec![Value::Int(20)]));
    h.alloc(GCObject::closure(vec![Value::Int(30)]));
    h.alloc(GCObject::closure(vec![Value::Int(40)]));
    h.free(1);
    h.free(2);
    h.alloc(GCObject::closure(vec![Value::Int(50)]));
    h.alloc(GCObject::closure(vec![Value::Int(60)]));
    h.dump();
}
