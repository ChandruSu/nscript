use ns::heap::heap;
use ns::vm::vm::Value;

#[test]
pub fn test_heap() {
    let mut h = heap::Heap::new(4);
    h.alloc(heap::GCObject::Closure(vec![Value::Int(10)]));
    h.alloc(heap::GCObject::Closure(vec![Value::Int(20)]));
    h.alloc(heap::GCObject::Closure(vec![Value::Int(30)]));
    h.alloc(heap::GCObject::Closure(vec![Value::Int(40)]));
    h.free(2);
    h.alloc(heap::GCObject::Closure(vec![Value::Int(50)]));
    h.alloc(heap::GCObject::Closure(vec![Value::Int(60)]));
    h.dump();
}
