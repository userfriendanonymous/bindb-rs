use std::{fs::File, path::Path};
pub use super::dynamic::EntryId as DynamicEntryId;

type IndexData = DynamicEntryId;

#[derive(Debug)]
pub enum AddError {
    RawAdd(super::dynamic::AddError),
    AddIndex(super::fixed::AddError),
    RemoveFreeId(super::fixed::RemoveLastError)
}

#[derive(Debug)]
pub enum RemoveError {
    RawRemove(super::dynamic::RemoveError),
    AddIndex(super::fixed::AddError),
    RemoveLastIndex(super::fixed::RemoveLastError),
    AddFreeId(super::fixed::AddError),
}

#[derive(Debug)]
pub enum OpenError {
    // FixedOpen(super::fixed::OpenError),
}

// #[derive(Debug)]
// pub enum CreateError {
//     FixedCreate(super::fixed::CreateError),
// }

pub struct Value<E> {
    raw: super::Dynamic<E>,
    indices: super::Fixed<IndexData>,
    free_ids: super::Fixed<u64>,
}

impl<E: binbuf::Dynamic> Value<E> {
    // pub unsafe fn open_folder<P: AsRef<Path>>(dir: P) {
    //     Self::open(
    //         super::Dynamic::op,
    //         indices,
    //         free_ids
    //     )
    // }

    pub unsafe fn open(raw: super::Dynamic<E>, indices: super::Fixed<IndexData>, free_ids: super::Fixed<u64>) -> Result<Self, OpenError> {
        Ok(Self {
            raw,
            indices,
            free_ids
        })
    }

    pub fn is_id_valid(&self, id: u64) -> bool {
        id < self.indices.len()
    }

    // pub unsafe fn create(raw: super::Dynamic<E>, indices_file: File, free_ids_file: File) -> Result<Self, CreateError> {
    //     Ok(Self {
    //         raw,
    //         indices: super::Fixed::create(indices_file).map_err(CreateError::FixedCreate)?,
    //         free_ids: super::Fixed::create(free_ids_file).map_err(CreateError::FixedCreate)?,
    //     })
    // }

    pub unsafe fn buf_unchecked(&self, id: u64) -> binbuf::BufConst<E> {
        let raw_id = binbuf::fixed::decode::<IndexData, _>(self.indices.buf_unchecked(id));
        self.raw.buf_unchecked(raw_id)
    }

    pub unsafe fn buf_mut_unchecked(&mut self, id: u64) -> binbuf::BufMut<E> {
        let raw_id = binbuf::fixed::decode::<IndexData, _>(self.indices.buf_unchecked(id));
        self.raw.buf_mut_unchecked(raw_id)
    }

    pub fn buf(&self, id: u64) -> binbuf::BufConst<E> {
        if self.is_id_valid(id) {
            unsafe { self.buf_unchecked(id) }
        } else {
            panic!("Id is invalid: {id}");
        }
    }

    pub fn buf_mut(&mut self, id: u64) -> binbuf::BufMut<E> {
        if self.is_id_valid(id) {
            unsafe { self.buf_mut_unchecked(id) }
        } else {
            panic!("Id is invalid: {id}");
        }
    }

    pub fn add(&mut self, value: impl binbuf::dynamic::Readable<E>) -> Result<u64, AddError> {
        let raw_id = self.raw.add(value).map_err(AddError::RawAdd)?;
        let id = if let Some(id_buf) = self.free_ids.last_buf() {
            let id = binbuf::fixed::decode::<u64, _>(id_buf);
            self.free_ids.remove_last().map_err(AddError::RemoveFreeId)?;
            self.indices.set(id, &raw_id);
            id
        } else {
            self.indices.add(&raw_id).map_err(AddError::AddIndex)?
        };
        Ok(id)
    }

    pub unsafe fn remove(&mut self, id: u64) -> Result<(), RemoveError> {
        let raw_id = binbuf::fixed::decode::<IndexData, _>(self.indices.buf_unchecked(id));
        println!("remove raw_id");
        self.raw.remove(raw_id).map_err(RemoveError::RawRemove)?;
        println!("raw removed");

        if self.indices.remove_if_last(id).map_err(RemoveError::RemoveLastIndex)? {
            self.free_ids.add(&id).map_err(RemoveError::AddFreeId)?;
            println!("free_ids added");
        }

        // if id == self.indices.last_id() {
        //     self.indices.remove_last().map_err(RemoveError::RemoveLastIndex)?;
        //     println!("indices removed last");
        // } else {
        //     self.free_ids.add(&id).map_err(RemoveError::AddFreeId)?;
        //     println!("free_ids added");
        // }
        Ok(())
    }
}

impl<E: binbuf::dynamic::Decode> Value<E> {
    pub fn get(&self, id: u64) -> E {
        binbuf::dynamic::decode(self.buf(id)).0
    }
}