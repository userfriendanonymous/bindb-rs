use crate::{
    entry,
    utils::{slice_to_array, slice_to_array_mut},
    Entry,
};
use std::{fs::File, path::Path};
// use super::{entry, buf, Codable, Lens, AsBuf};
use memmap2::{MmapAsRawDesc, MmapMut};

#[derive(Debug)]
pub enum GetError {
    InvalidId,
}

#[derive(Debug)]
pub enum AddError {
    Fmmap(fmmap::error::Error),
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum RemoveLastError {
    Fmmap(fmmap::error::Error),
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum SwapRemoveError {
    RemoveLastError(RemoveLastError),
    Fmmap(fmmap::error::Error),
}

#[derive(Debug)]
pub enum NewError {
    Fmmap(fmmap::error::Error),
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum OpenError {
    Io(std::io::Error),
}

#[derive(Clone, Debug)]
pub struct Meta<E> {
    next_entry_id: entry::Id<E>,
}

pub struct Value<E> {
    file: File,
    next_entry_id: entry::Id<E>,
    file_map: MmapMut,
    margin: u64,
    max_margin: u64,
}

impl<E: Entry> Value<E> {
    pub unsafe fn new<F: MmapAsRawDesc>(path: F) -> Result<Self, NewError> {
        type E = NewError;
        let file = File::create(&path).map_err(E::Io)?;
        file.set_len(8).map_err(E::Io)?;
        let mut file_map = MmapMut::map_mut(file);

        let next_entry_id = entry::Id::zero();
        file_map
            .write_u64(next_entry_id.as_u64(), 0)
            .map_err(E::Fmmap)?;
        Ok(Self {
            margin: 0,
            max_margin: 1000,
            file,
            file_map,
            next_entry_id,
        })
    }

    pub unsafe fn open(file: &File) -> Result<Self, OpenError> {
        type E = OpenError;
        let file_map = MmapMut::map_mut(file).map_err(E::Io)?;
        let next_entry_id = entry::Id::from_u64(file_map.read_u64(0).map_err(E::Fmmap)?);
        Ok(Self {
            margin: 0,
            max_margin: 1000,
            file,
            file_map,
            next_entry_id,
        })
    }

    fn entry_offset(&self, id: entry::Id<E>) -> usize {
        8 + Entry::LEN * id.as_usize()
    }

    pub fn last_entry_id(&self) -> Option<entry::Id<E>> {
        match self.next_entry_id.as_u64() {
            0 => None,
            _ => Some(self.next_entry_id - 1),
        }
    }

    // region: Core functions.
    pub unsafe fn buf_unchecked(&self, id: entry::Id<E>) -> entry::BufConst<'_, E> {
        let offset = self.entry_offset(id);
        let bytes = entry::Bytes::new(self.file_map.get_unchecked(offset..offset + Entry::LEN));
        E::buf(bytes)
    }

    pub unsafe fn buf_unchecked_mut(&self, id: entry::Id<Entry>) -> buf::Mut<'_, Entry> {
        let offset = self.entry_offset(id);
        let bytes = self
            .file_map
            .as_slice()
            .get_unchecked_mut(offset..offset + Entry::LEN);
        Ok(Entry::buf(buf::bytes::Mut::new(bytes)))
    }

    pub fn buf(&self, id: entry::Id<Entry>) -> Option<buf::Ref<'_, Entry>> {
        if self.next_entry_id > id {
            Some(unsafe { self.buf_unchecked(id) })
        } else {
            None
        }
    }

    pub fn buf_mut(&mut self, id: entry::Id<Entry>) -> Option<buf::Mut<'_, Entry>> {
        if self.next_entry_id > id {
            Some(unsafe { self.buf_unchecked_mut(id) })
        } else {
            None
        }
    }
    // endregion: Core functions.

    pub fn add(&mut self, entry: &impl buf::Write<Entry>) -> Result<entry::Id<Entry>, AddError> {
        type E = AddError;
        let id = self.next_entry_id;
        if self.margin == 0 {
            let new_size = self.entry_offset(id + self.max_margin + 2) as u64;
            self.file.set_len(new_size).map_err(E::Io)?;
            self.file_map.truncate(new_size).map_err(E::Fmmap)?;
            self.margin = self.max_margin + 1;
        }
        self.margin -= 1;
        self.set(Lens::FULL, id, entry).map_err(E::Fmmap)?;
        self.next_entry_id = self.next_entry_id.succ();
        self.file_map
            .write_u64(self.next_entry_id.as_u64(), 0)
            .map_err(E::Fmmap)?;
        Ok(id)
    }

    pub fn remove_last(&mut self) -> Result<(), RemoveLastError> {
        type E = RemoveLastError;
        let id = self.next_entry_id.prev();
        if self.margin >= self.max_margin {
            let new_size = self.entry_offset(id) as u64;
            self.file.set_len(new_size).map_err(E::Io)?;
            self.file_map.truncate(new_size).map_err(E::Fmmap)?;
            self.margin = 0;
        }
        self.margin += 1;
        self.file_map
            .write_u64(self.next_entry_id.as_u64(), 0)
            .map_err(E::Fmmap)?;
        Ok(())
    }

    pub fn all_ids(&self) -> impl Iterator<Item = entry::Id<Entry>> {
        entry::id::Range(entry::Id::zero(), self.next_entry_id)
    }

    // Convenience functions.
    pub fn get<O: Buf>(&self, lens: impl Lens<Entry, O>, id: entry::Id<Entry>) -> Option<O>
    where
        O: buf::Decode,
    {
        let buf = lens.view(self.buf(id)?);
        Some(O::decode(&bytes))
    }

    pub fn set(
        &'a mut self,
        lens: Lens<Entry, T>,
        id: entry::Id<Entry>,
        value: &impl buf::Write<T>,
    ) -> Result<(), fmmap::error::Error>
    where
        [(); T::SIZE]:,
    {
        let mut buf_mut = self.buf_mut(lens, id)?;
        buf_mut.set(value);
        Ok(())
    }

    pub fn find<T: Codable>(
        &self,
        lens: Lens<Entry, T>,
        // ids: impl Iterator<Item = entry::Id<Entry>>,
        f: impl Fn(&T) -> bool,
    ) -> Result<Option<(entry::Id<Entry>, T)>, GetError>
    where
        [(); T::SIZE]:,
    {
        for id in self.all_ids() {
            let data = self.get(lens, id)?;
            if f(&data) {
                return Ok(Some((id, data)));
            }
        }
        Ok(None)
    }

    pub fn find_exact<'a, T: Codable>(
        &self,
        lens: Lens<Entry, T>,
        // ids: impl Iterator<Item = entry::Id<Entry>>,
        other: &'a impl AsBuf<'a, T>,
    ) -> Result<Option<entry::Id<Entry>>, fmmap::error::Error>
    where
        [(); T::SIZE]:,
    {
        let buf = other.as_buf();
        let buf = buf.to_ref();
        for id in self.all_ids() {
            let data = self.buf_ref(lens, id)?;
            if buf == data {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    pub fn copy<T: Codable>(
        &mut self,
        lens: Lens<Entry, T>,
        src_id: entry::Id<Entry>,
        dst_id: entry::Id<Entry>,
    ) -> Result<(), fmmap::error::Error>
    where
        [(); T::SIZE]:,
    {
        let mut dst = unsafe { self.buf_mut(lens, dst_id)?.detach() };
        let src = self.buf_ref(lens, src_id)?;
        dst.set(&src);
        Ok(())
    }

    pub fn swap<T: Codable>(
        &mut self,
        lens: Lens<Entry, T>,
        a_id: entry::Id<Entry>,
        b_id: entry::Id<Entry>,
    ) -> Result<(), fmmap::error::Error>
    where
        [(); T::SIZE]:,
    {
        if a_id != b_id {
            let mut a = unsafe { self.buf_mut(lens, a_id)?.detach() };
            let mut b = self.buf_mut(lens, b_id)?;
            a.swap(&mut b);
        }
        Ok(())
    }

    /// Swaps given entry with the last entry and removes it.
    /// WARNING: This method changes ID of the last entry in this collection.
    /// You probably should only use this if this collection is used as a stack/set and not as a map.
    pub fn swap_remove(&mut self, id: entry::Id<Entry>) -> Result<(), SwapRemoveError> {
        type E = SwapRemoveError;
        if let Some(last_entry_id) = self.last_entry_id() {
            self.swap(Lens::FULL, id, last_entry_id).map_err(E::Fmmap)?;
        }
        self.remove_last().map_err(E::RemoveLastError)?;
        Ok(())
    }
}
