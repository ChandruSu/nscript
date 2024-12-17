pub mod gc {
    use std::collections::HashMap;

    use crate::vm::vm;

    struct GCObject {
        marked: bool,
        children: Vec<usize>,
        values: HashMap<vm::Value, vm::Value>,
    }

    pub struct Arena {
        all: Vec<GCObject>,
    }

    impl GCObject {
        pub fn new() -> Self {
            Self {
                marked: true,
                values: HashMap::new(),
                children: vec![],
            }
        }
    }

    impl Arena {
        pub fn new() -> Self {
            Self { all: vec![] }
        }

        pub fn alloc(&mut self) -> vm::Value {
            let id = self.all.len() as u32;
            self.all.push(GCObject::new());
            vm::Value::Object(id)
        }

        pub fn set_obj_mark(&mut self, id: usize, status: bool) {
            if self.all[id].marked == status {
                return;
            }

            self.all[id].marked = status;

            for child in self
                .get_obj(id)
                .children
                .iter()
                .copied()
                .collect::<Vec<usize>>()
            {
                self.set_obj_mark(child, status);
            }
        }

        pub fn get_obj(&mut self, id: usize) -> &GCObject {
            &self.all[id]
        }

        pub fn get_obj_mut(&mut self, id: usize) -> &mut GCObject {
            &mut self.all[id]
        }
    }
}
