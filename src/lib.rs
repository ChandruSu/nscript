mod backend;
pub mod error;
mod frontend;
mod interpreter;
mod utils;
mod vm;

pub use interpreter::Interpreter;
pub use vm::heap::Alloc;
pub use vm::heap::HeapNode;
pub use vm::Env;
pub use vm::ModuleFnRecord;
pub use vm::NativeFnPtr;
pub use vm::Value;
