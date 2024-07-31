pub use fixed::Value as Fixed;
pub use dynamic::Value as Dynamic;
pub use indexed_dynamic::Value as IndexedDynamic;
pub use single::Value as Single;
pub use binary_tree::Value as BinaryTree;

pub mod fixed;
pub mod dynamic;
pub mod indexed_dynamic;
pub mod binary_tree;
pub mod single;

#[derive(Clone, Copy, Debug)]
pub enum OpenMode {
    New,
    Existing,
}