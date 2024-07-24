use std::{fmt::Debug, fs::File};
use binbuf::{bytes_ptr, fixed::Readable, impls::{arb_num, ArbNum}, BytesPtr, Entry, Fixed as _};

mod search;

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
pub enum NodeBranch {
    Left,
    Right
}

#[derive(Debug)]
pub enum AddError {
    AddNode(super::fixed::AddError),
    RemoveLastFreeId(super::fixed::RemoveLastError)
}

#[derive(Debug)]
pub enum RemoveError {
    RemoveNode(RemoveNodeError),
}

#[derive(Debug)]
pub enum RemoveNodeError {
    RemoveIfLast(super::fixed::RemoveLastError),
    AddFreeId(super::fixed::AddError),
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

pub struct Searched {
    parent: Option<NodeParent>,
    id: Option<u64>
}

impl Searched {
    pub fn is_found(&self) -> bool {
        self.id.is_some()
    }

    pub fn find(self) -> Result<SearchedFound, SearchedNotFound> {
        match self.id {
            Some(id) => Ok(SearchedFound { parent: self.parent, id }),
            None => Err(SearchedNotFound { parent: self.parent })
        }
    }
}

pub struct SearchedFound {
    parent: Option<NodeParent>,
    id: u64,
}

pub struct SearchedNotFound {
    parent: Option<NodeParent>,
}

#[derive(Clone, Copy)]
pub struct NodeParent {
    id: u64,
    branch: NodeBranch,
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

    pub fn search(&self, key: impl binbuf::fixed::BufOrd<K> + Clone) -> Searched {
        // let mut arr = [0u8; T::LEN];
        // let key_buf = unsafe { T::buf(bytes_ptr::Mut::from_slice(&mut arr)) };
        // key.write_to(key_buf);
        let mut parent = None;
        let Some(mut node_id) = self.root_id else {
            return Searched { id: None, parent: None };
        };
        loop {
            let node = unsafe { self.node_buf_by_id(node_id) };
            match key.clone().buf_cmp(Node::buf_key(node)) {
                std::cmp::Ordering::Less => {
                    let left_id = binbuf::fixed::decode::<I, _>(Node::buf_left_id(node)).to_u64();
                    parent = Some(NodeParent { id: node_id, branch: NodeBranch::Left });
                    if left_id == 0 {
                        return Searched { id: None, parent };
                        // return Err(Some((node_id, NodeBranch::Left)));
                    } else {
                        // parent = Some((node_id, NodeBranch::Left));
                        node_id = left_id - 1;
                    }
                },
                std::cmp::Ordering::Equal => {
                    return Searched { parent, id: Some(node_id) };
                },
                std::cmp::Ordering::Greater => {
                    let right_id = binbuf::fixed::decode::<I, _>(Node::buf_right_id(node)).to_u64();
                    parent = Some(NodeParent { id: node_id, branch: NodeBranch::Right });
                    if right_id == 0 {
                        return Searched { id: None, parent };
                        // return Err(Some((node_id, NodeBranch::Right)));
                    } else {
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

    pub unsafe fn buf_searched(&self, searched: &SearchedFound) -> binbuf::BufConst<V> {
        Node::buf_value(unsafe { self.node_buf_by_id(searched.id) })
    }

    pub unsafe fn buf_mut_searched(&mut self, searched: &SearchedFound) -> binbuf::BufMut<V> {
        Node::buf_value(unsafe { self.node_buf_mut_by_id(searched.id) })
    }

    pub fn buf(&self, key: impl binbuf::fixed::BufOrd<K> + Clone) -> Option<binbuf::BufConst<V>> {
        self.search(key).find().ok().map(|s| unsafe { self.buf_searched(&s) })
    }

    pub fn buf_mut(&mut self, key: impl binbuf::fixed::BufOrd<K> + Clone) -> Option<binbuf::BufMut<V>> {
        self.search(key).find().ok().map(|s| unsafe { self.buf_mut_searched(&s) })
    }

    // Returns true if item already exists.
    pub unsafe fn add_searched(
        &mut self,
        search: &SearchedNotFound,
        key: impl binbuf::fixed::BufOrd<K> + Clone,
        value: impl binbuf::fixed::Readable<V>
    ) -> Result<(), AddError>
    where [(); Node::<I, K, V>::LEN]: {
        let mut node_arr = [0u8; Node::<I, K, V>::LEN];
        let node_buf = unsafe { Node::buf(bytes_ptr::Mut::from_slice(&mut node_arr)) };
        key.clone().write_to(Node::<I, K, V>::buf_key(node_buf));
        value.write_to(Node::<I, K, V>::buf_value(node_buf));
        I::from_u64(0u64).encode(Node::<I, K, V>::buf_left_id(node_buf));
        I::from_u64(0u64).encode(Node::<I, K, V>::buf_right_id(node_buf));

        match search.parent {
            None => {
                let id = self.nodes.add(node_buf).map_err(AddError::AddNode)?;
                self.set_root_id(Some(id));
            }
            Some(parent) => {
                let id = match self.free_ids.last_buf() {
                    Some(id_buf) => {
                        let id = binbuf::fixed::decode::<u64, _>(id_buf);
                        node_buf.write_to(unsafe { self.node_buf_mut_by_id(id) });
                        self.free_ids.remove_last().map_err(AddError::RemoveLastFreeId)?;
                        id
                    },
                    None => {
                        self.nodes.add(node_buf).map_err(AddError::AddNode)?
                    }
                };
                println!("ADDING: id = {id}");

                let parent_buf = unsafe { self.node_buf_mut_by_id(parent.id) };
                match parent.branch {
                    NodeBranch::Left => {
                        I::from_u64(id + 1).write_to(Node::<I, K, V>::buf_left_id(parent_buf));
                    },
                    NodeBranch::Right => {
                        I::from_u64(id + 1).write_to(Node::<I, K, V>::buf_right_id(parent_buf));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn add(&mut self, key: impl binbuf::fixed::BufOrd<K> + Clone, value: impl binbuf::fixed::Readable<V>) -> Result<bool, AddError>
    where [(); Node::<I, K, V>::LEN]: {
        match self.search(key.clone()).find() {
            Ok(_) => Ok(true),
            Err(s) => {
                unsafe { self.add_searched(&s, key, value) }?;
                Ok(false)
            }
        }
    }

    fn remove_node(&mut self, id: u64) -> Result<(), RemoveNodeError> {
        if self.nodes.remove_if_last(id).map_err(RemoveNodeError::RemoveIfLast)? {
            self.free_ids.add(&id).map_err(RemoveNodeError::AddFreeId)?;
        }
        Ok(())
    }

    // Returns true if item doesn't exist.
    pub unsafe fn remove_searched(&mut self, searched: &SearchedFound) -> Result<(), RemoveError> {
        println!("--- REMOVING NODE --- root_id = {:?}", self.root_id);

        let node_buf = unsafe { self.node_buf_mut_by_id(searched.id) };
        let left_id = binbuf::fixed::decode::<I, _>(Node::buf_left_id(node_buf)).to_u64();
        let right_id = binbuf::fixed::decode::<I, _>(Node::buf_right_id(node_buf)).to_u64();
        println!("Entry node: id = {}, left_id = {left_id}, right_id = {right_id}", searched.id);
        
        match (left_id, right_id, searched.parent) {
            (_, _, Some(parent)) if left_id == 0 || right_id == 0 => {
                println!("Entry has parent and at most single branch");
                self.remove_node(searched.id).map_err(RemoveError::RemoveNode)?;

                let connect_id = if left_id == 0 { right_id } else { left_id };
                println!("connect_id = {connect_id}");
                let mut parent_buf = unsafe { self.node_buf_mut_by_id(parent.id) };
                match parent.branch {
                    NodeBranch::Left => {
                        I::from_u64(connect_id).encode(Node::<I, K, V>::buf_left_id(parent_buf));
                    }
                    NodeBranch::Right => {
                        I::from_u64(connect_id).encode(Node::<I, K, V>::buf_right_id(parent_buf));
                    }
                }
            },
            (0, 0, None) => {
                println!("Entry is root with no branches");
                self.remove_node(searched.id).map_err(RemoveError::RemoveNode)?;
                self.set_root_id(None);
            },
            (_, _, None) if left_id == 0 || right_id == 0 => {
                println!("Entry is root with at most single branch");
                self.remove_node(searched.id).map_err(RemoveError::RemoveNode)?;

                let root_id = if left_id == 0 { right_id } else { left_id } - 1;
                self.set_root_id(Some(root_id));
            },

            // The most complex case to handle: both left and right branches exist.
            (_, _, parent) => {
                println!("Entry has both branches");
                debug_assert_ne!(left_id, 0);
                debug_assert_ne!(right_id, 0);

                self.remove_node(searched.id).map_err(RemoveError::RemoveNode)?;

                let mut node_parent_id = searched.id;
                let mut node_id = right_id - 1;
                let mut idx = 0;
                loop {
                    idx += 1;
                    if idx > 20 {
                        panic!("Loop stuck!");
                    }

                    let node_buf = unsafe { self.node_buf_mut_by_id(node_id) };
                    let node_left_id_buf = Node::<I, K, V>::buf_left_id(node_buf);
                    let node_left_id = binbuf::fixed::decode::<I, _>(node_left_id_buf).to_u64();
                    let node_right_id_buf = Node::<I, K, V>::buf_right_id(node_buf);
                    let node_right_id = binbuf::fixed::decode::<I, _>(node_right_id_buf).to_u64();
                    println!("Testing node_id = {}, (left_id = {node_left_id}, right_id = {node_right_id})", node_id + 1);
                    if node_left_id == 0 {
                        I::from_u64(node_right_id)
                            .write_to(Node::buf_left_id(unsafe { self.node_buf_mut_by_id(node_parent_id) }));

                        I::from_u64(left_id).write_to(node_left_id_buf);
                        if searched.id != node_parent_id {
                            I::from_u64(right_id).write_to(node_right_id_buf);
                        }

                        match parent {
                            Some(parent) => {
                                println!("Parent exists.");
                                let parent_buf = unsafe { self.node_buf_mut_by_id(parent.id) };
                                match parent.branch {
                                    NodeBranch::Left => {
                                        I::from_u64(node_id + 1).encode(Node::<I, K, V>::buf_left_id(parent_buf));
                                    }
                                    NodeBranch::Right => {
                                        I::from_u64(node_id + 1).encode(Node::<I, K, V>::buf_right_id(parent_buf));
                                    }
                                }
                            },
                            None => {
                                println!("Parent doesn't exist.");
                                self.set_root_id(Some(node_id));
                            }
                        }

                        break;
                    } else {
                        println!("Test node left id != 0. Continue");
                        node_parent_id = node_id;
                        node_id = node_left_id - 1;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn remove(&mut self, key: impl binbuf::fixed::BufOrd<K> + Clone) -> Result<bool, RemoveError> {
        match self.search(key).find() {
            Ok(s) => {
                unsafe { self.remove_searched(&s) }?;
                Ok(false)
            },
            Err(_) => Ok(true)
        }
    }
}

impl<I: NodeId, K: binbuf::fixed::Decode + Debug, V: binbuf::fixed::Decode> Value<I, K, V> {
    pub fn get(&self, key: impl binbuf::fixed::BufOrd<K> + Clone) -> Option<V> {
        self.search(key).find().ok().map(|s| unsafe { self.get_searched(&s) })
    }

    pub unsafe fn get_searched(&self, searched: &SearchedFound) -> V {
        V::decode(self.buf_searched(searched))
    }
}