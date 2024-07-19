use std::{fmt::Debug, fs::File};
use binbuf::{bytes_ptr, fixed::Readable, impls::{arb_num, ArbNum}, BytesPtr, Entry, Fixed as _};

pub trait NodeId: binbuf::fixed::Decode {
    fn to_u64(self) -> u64;
    fn from_u64(value: u64) -> Self;
}

impl<const LEN: usize> NodeId for ArbNum<LEN, u64> {
    fn from_u64(value: u64) -> Self {
        ArbNum::new(value)
    }
    fn to_u64(self) -> u64 {
        self.unwrap()
    }
}

impl NodeId for u64 {
    fn from_u64(value: u64) -> Self {
        value
    }
    fn to_u64(self) -> u64 {
        self
    }
}

binbuf::fixed! {
    pub struct Node<I: NodeId, K, V> {
        #[lens(buf_key)]
        key: K,
        #[lens(buf_value)]
        value: V,
        #[lens(buf_left_id)]
        left_id: I,
        #[lens(buf_right_id)]
        right_id: I,
    }
    buf! { pub struct NodeBuf<P, I: NodeId, K: binbuf::Fixed, V: binbuf::Fixed>(Node<I, K, V>, P); }

    impl<I: NodeId, K: binbuf::Fixed, V: binbuf::Fixed> I for Node<I, K, V> {
        type Buf<P> = NodeBuf<P, I, K, V>;
    }

    impl<I: NodeId, K: binbuf::Fixed, V: binbuf::Fixed> Encode for Node<I, K, V> {}
    impl<I: NodeId, K: binbuf::fixed::Decode, V: binbuf::fixed::Decode> Decode for Node<I, K, V> {}
}

binbuf::fixed! {
    pub struct Header {
        #[lens(buf_root_id)]
        root_id: Option<u64>,
    }
    buf! { pub struct HeaderBuf<P>(Header, P); }

    impl I for Header {
        type Buf<P> = HeaderBuf<P>;
    }
    impl Code for Header {}
}

#[derive(Clone, Copy, Debug)]
pub enum NodeChildType {
    Left,
    Right
}

#[derive(Debug)]
pub enum AddError {
    AddNode(super::fixed::AddError)
}

#[derive(Debug)]
pub enum RemoveError {
    RemoveNode(super::fixed::RemoveLastError)
}

#[derive(Debug)]
pub enum CreateError {
    CreateHeader(super::single::CreateError),
    NodesNotEmpty,
    FreeIdsNotEmpty,
}

#[derive(Debug)]
pub enum OpenError {

}

pub struct Value<I: NodeId, K, V> {
    nodes: super::Fixed<Node<I, K, V>>,
    free_ids: super::Fixed<u64>,
    header: super::Single<Header>,
    root_id: Option<u64>,
}

impl<I: NodeId, K: binbuf::fixed::Decode + Debug, V: binbuf::Fixed> Value<I, K, V> {
    // Id must not be 0!

    pub unsafe fn create(
        nodes: super::Fixed<Node<I, K, V>>,
        free_ids: super::Fixed<u64>,
        header_file: File,
    ) -> Result<Self, CreateError> {
        if nodes.len() != 0 { Err(CreateError::NodesNotEmpty)? }
        if nodes.len() != 0 { Err(CreateError::FreeIdsNotEmpty)? }
        let header = super::Single::create(
            header_file,
            &Header { root_id: None })
            .map_err(CreateError::CreateHeader)?;
        Ok(Self {
            nodes,
            free_ids,
            header,
            root_id: None,
        })
    }

    pub unsafe fn open(
        nodes: super::Fixed<Node<I, K, V>>,
        free_ids: super::Fixed<u64>,
        header: super::Single<Header>,
    ) -> Result<Self, OpenError> {
        let root_id = header.get().root_id;
        Ok(Self {
            nodes,
            free_ids,
            header,
            root_id,
        })
    }

    unsafe fn node_buf_by_id(&self, id: u64) -> binbuf::BufConst<Node<I, K, V>> {
        self.nodes.buf_unchecked(id)
    }

    unsafe fn node_buf_mut_by_id(&mut self, id: u64) -> binbuf::BufMut<Node<I, K, V>> {
        self.nodes.buf_mut_unchecked(id)
    }

    fn search(&self, key: impl binbuf::fixed::BufOrd<K> + Clone) -> Result<(u64, Option<(u64, NodeChildType)>), Option<(u64, NodeChildType)>> {
        // let mut arr = [0u8; T::LEN];
        // let key_buf = unsafe { T::buf(bytes_ptr::Mut::from_slice(&mut arr)) };
        // key.write_to(key_buf);
        let mut parent = None;
        let mut node_id = self.root_id.ok_or(None)?;
        loop {
            let node = unsafe { self.node_buf_by_id(node_id) };
            match key.clone().buf_cmp(Node::buf_key(node)) {
                std::cmp::Ordering::Less => {
                    let left_id = binbuf::fixed::decode::<I, _>(Node::buf_left_id(node)).to_u64();
                    if left_id == 0 {
                        return Err(Some((node_id, NodeChildType::Left)));
                    } else {
                        parent = Some((node_id, NodeChildType::Left));
                        node_id = left_id - 1;
                    }
                },
                std::cmp::Ordering::Equal => {
                    return Ok((node_id, parent));
                },
                std::cmp::Ordering::Greater => {
                    let right_id = binbuf::fixed::decode::<I, _>(Node::buf_right_id(node)).to_u64();
                    if right_id == 0 {
                        return Err(Some((node_id, NodeChildType::Right)));
                    } else {
                        parent = Some((node_id, NodeChildType::Right));
                        node_id = right_id - 1;
                    }
                },
            }
        }
    }

    fn set_root_id(&mut self, id: Option<u64>) {
        id.encode(Header::buf_root_id(self.header.buf_mut()));
        self.root_id = id;
    }

    pub fn buf(&self, key: impl binbuf::fixed::BufOrd<K> + Clone) -> Option<binbuf::BufConst<V>> {
        match self.search(key) {
            Ok((id, _)) => Some(Node::buf_value(unsafe { self.node_buf_by_id(id) })),
            Err(_) => None
        }
    }

    pub fn buf_mut(&mut self, key: impl binbuf::fixed::BufOrd<K> + Clone) -> Option<binbuf::BufMut<V>> {
        match self.search(key) {
            Ok((id, _)) => Some(Node::buf_value(unsafe { self.node_buf_mut_by_id(id) })),
            Err(_) => None
        }
    }

    pub fn add(&mut self, key: impl binbuf::fixed::BufOrd<K> + Clone, value: impl binbuf::fixed::Readable<V>) -> Result<(), AddError>
    where [(); Node::<I, K, V>::LEN]: {
        let mut node_arr = [0u8; Node::<I, K, V>::LEN];
        let node_buf = unsafe { Node::buf(bytes_ptr::Mut::from_slice(&mut node_arr)) };
        key.clone().write_to(Node::<I, K, V>::buf_key(node_buf));
        value.write_to(Node::<I, K, V>::buf_value(node_buf));
        I::from_u64(0u64).encode(Node::<I, K, V>::buf_left_id(node_buf));
        I::from_u64(0u64).encode(Node::<I, K, V>::buf_right_id(node_buf));

        match self.search(key) {
            Ok(_) => panic!("Node with same key already exists"),
            Err(None) => {
                let id = self.nodes.add(node_buf).map_err(AddError::AddNode)?;
                self.set_root_id(Some(id));
            }
            Err(Some((parent_id, child_type))) => {
                let id = match self.free_ids.last_buf() {
                    Some(id_buf) => {
                        let id = binbuf::fixed::decode::<u64, _>(id_buf);
                        node_buf.write_to(unsafe { self.node_buf_mut_by_id(id) });
                        id
                    },
                    None => {
                        self.nodes.add(node_buf).map_err(AddError::AddNode)?
                    }
                };

                let parent_buf = unsafe { self.node_buf_mut_by_id(parent_id) };
                match child_type {
                    NodeChildType::Left => {
                        I::from_u64(id + 1).write_to(Node::<I, K, V>::buf_left_id(parent_buf));
                    },
                    NodeChildType::Right => {
                        I::from_u64(id + 1).write_to(Node::<I, K, V>::buf_right_id(parent_buf));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn remove(&mut self, key: impl binbuf::fixed::BufOrd<K> + Clone) -> Result<(), RemoveError> {
        match self.search(key) {
            Err(_) => panic!("Node with such key doesn't exist"),
            Ok((id, parent)) => {
                let node_buf = unsafe { self.node_buf_mut_by_id(id) };
                let left_id = binbuf::fixed::decode::<I, _>(Node::buf_left_id(node_buf)).to_u64();
                let right_id = binbuf::fixed::decode::<I, _>(Node::buf_right_id(node_buf)).to_u64();
                match (left_id, right_id, parent) {
                    (_, _, Some((parent_id, child_type))) if left_id == 0 || right_id == 0 => {
                        if self.nodes.remove_if_last(id).map_err(RemoveError::RemoveNode)? {
                            self.free_ids.add(&id);
                        }

                        let connect_id = if left_id == 0 { right_id } else { left_id };
                        let mut parent_buf = unsafe { self.node_buf_mut_by_id(parent_id) };
                        match child_type {
                            NodeChildType::Left => {
                                I::from_u64(connect_id).encode(Node::<I, K, V>::buf_left_id(parent_buf));
                            }
                            NodeChildType::Right => {
                                I::from_u64(connect_id).encode(Node::<I, K, V>::buf_right_id(parent_buf));
                            }
                        }
                    },
                    (0, 0, None) => {
                        if self.nodes.remove_if_last(id).map_err(RemoveError::RemoveNode)? {
                            self.free_ids.add(&id);
                        }
                        self.set_root_id(None);
                    },
                    (_, _, None) if left_id == 0 || right_id == 0 => {
                        if self.nodes.remove_if_last(id).map_err(RemoveError::RemoveNode)? {
                            self.free_ids.add(&id);
                        }

                        let root_id = if left_id == 0 { right_id } else { left_id } - 1;
                        self.set_root_id(Some(root_id));
                    },

                    // The most complex case to handle: both left and right branches exist.
                    (_, _, parent) => {
                        debug_assert_ne!(left_id, 0);
                        debug_assert_ne!(right_id, 0);

                        if self.nodes.remove_if_last(id).map_err(RemoveError::RemoveNode)? {
                            self.free_ids.add(&id);
                        }

                        let mut node_parent_id = id;
                        let mut node_id = right_id - 1;
                        loop {
                            let node_buf = unsafe { self.node_buf_mut_by_id(node_id) };
                            let node_left_id_buf = Node::<I, K, V>::buf_left_id(node_buf);
                            let node_left_id = binbuf::fixed::decode::<I, _>(node_left_id_buf).to_u64();
                            if node_left_id == 0 {
                                I::from_u64(left_id).write_to(node_left_id_buf);
                                I::from_u64(right_id).write_to(Node::<I, K, V>::buf_right_id(node_buf));

                                I::from_u64(0).write_to(Node::buf_left_id(unsafe { self.node_buf_mut_by_id(node_parent_id) }));

                                match parent {
                                    Some((parent_id, child_type)) => {
                                        let parent_buf = unsafe { self.node_buf_mut_by_id(parent_id) };
                                        match child_type {
                                            NodeChildType::Left => {
                                                I::from_u64(node_id).encode(Node::<I, K, V>::buf_left_id(parent_buf));
                                            }
                                            NodeChildType::Right => {
                                                I::from_u64(node_id).encode(Node::<I, K, V>::buf_right_id(parent_buf));
                                            }
                                        }
                                    },
                                    None => {
                                        self.set_root_id(Some(node_id));
                                    }
                                }

                                break;
                            } else {
                                node_parent_id = node_id;
                                node_id = node_left_id - 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl<I: NodeId, K: binbuf::fixed::Decode + Debug, V: binbuf::fixed::Decode> Value<I, K, V> {
    pub fn get(&self, key: impl binbuf::fixed::BufOrd<K> + Clone) -> Option<V> {
        self.buf(key).map(|value| V::decode(value))
    }
}