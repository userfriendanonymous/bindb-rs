use std::{fs::File, path::Path};
use super::OpenMode;

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
    DynamicOpen(super::dynamic::OpenError),
    FixedOpen(super::fixed::OpenError),
}

pub struct OpenFiles {
    pub raw_entries: File,
    pub raw_free_locations: File,
    pub indices: File,
    pub free_ids: File,
}

pub struct OpenMaxMargins {
    pub raw_entries: u64,
    pub raw_free_locations: u64,
    pub indices: u64,
    pub free_ids: u64,
}

pub struct OpenConfig {
    pub mode: OpenMode,
    pub files: OpenFiles,
    pub max_margins: OpenMaxMargins,
}

pub struct Value<E> {
    raw: super::Dynamic<E>,
    indices: super::Fixed<IndexData>,
    free_ids: super::Fixed<u64>,
}

impl<E: binbuf::Dynamic> Value<E> {
    pub unsafe fn open(OpenConfig { mode, files, max_margins }: OpenConfig) -> Result<Self, OpenError> {
        Ok(Self {
            raw: super::Dynamic::open(super::dynamic::OpenConfig {
                mode,
                files: super::dynamic::OpenFiles { entries: files.raw_entries, free_locations: files.raw_free_locations },
                max_margins: super::dynamic::OpenMaxMargins { entries: max_margins.raw_entries, free_locations: max_margins.raw_free_locations },
            }).map_err(OpenError::DynamicOpen)?,
            indices: super::Fixed::open(mode, files.indices, max_margins.indices).map_err(OpenError::FixedOpen)?,
            free_ids: super::Fixed::open(mode, files.free_ids, max_margins.free_ids).map_err(OpenError::FixedOpen)?,
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

    pub fn free_locations_len(&self) -> u64 {
        self.raw.free_locations_len()
    }

    pub unsafe fn remove(&mut self, id: u64) -> Result<(), RemoveError> {
        let raw_id = binbuf::fixed::decode::<IndexData, _>(self.indices.buf_unchecked(id));
        self.raw.remove(raw_id).map_err(RemoveError::RawRemove)?;

        if self.indices.remove_if_last(id).map_err(RemoveError::RemoveLastIndex)? {
            self.free_ids.add(&id).map_err(RemoveError::AddFreeId)?;
        }
        Ok(())
    }
}

impl<E: binbuf::dynamic::Decode> Value<E> {
    pub fn get(&self, id: u64) -> E {
        binbuf::dynamic::decode(self.buf(id)).0
    }
}