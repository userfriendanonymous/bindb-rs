use super::{NodeId, NodeParent};

pub struct Value<'a, I: NodeId, K, V> {
    parent: Option<NodeParent>,
    id: Option<u64>,
    handle: &'a mut super::Value<I, K, V>
}

impl<'a, I: NodeId, K, V> Value<'a, I, K, V> {
    
}

pub struct Found<'a, I: NodeId, K, V> {
    parent: Option<NodeParent>,
    id: u64,
    handle: &'a mut super::Value<I, K, V>
}

impl<'a, I: NodeId, K, V> Found<'a, I, K, V> {

}

pub struct NotFound<'a, I: NodeId, K, V> {
    parent: Option<NodeParent>,
    handle: &'a mut super::Value<I, K, V>
}

impl<'a, I: NodeId, K, V> NotFound<'a, I, K, V> {

}