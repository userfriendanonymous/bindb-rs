use crate::{
    entry::{self, Readable, Codable as _}, lens,
    utils::{slice_to_array, slice_to_array_mut},
    Entry, Lens,
};
use std::{fs::File, path::Path};
// use super::{entry, buf, Codable, Lens, AsBuf};
use memmap2::{MmapAsRawDesc, MmapMut, MmapOptions};
pub use header::Value as Header;

pub mod header;

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

pub struct Value<E, M: Entry> {
    header: Header<E, M>,
    file: File,
    file_map: MmapMut,
    margin: u64,
    max_margin: u64,
}

impl<E: Entry, M: Entry + entry::Codable> Value<E, M> {
    pub unsafe fn create(file: File, meta: M) -> Result<Self, NewError> {
        type E = NewError;
        file.set_len(8).map_err(E::Io)?;
        let mut file_map = MmapMut::map_mut(&file).map_err(E::Io)?;
        
        Ok(Self {
            header: Header {
                next_entry_id: entry::Id::zero(),
                meta,
            },
            margin: 0,
            max_margin: 1000,
            file,
            file_map,
        })
    }

    pub unsafe fn open(file: File) -> Result<Self, OpenError> {
        type E = OpenError;
        let file_map = MmapMut::map_mut(&file).map_err(E::Io)?;
        let bytes = entry::bytes::Const::new(file_map.get_unchecked(0 .. Header::<E, M>::len()));
        let header = Header::decode(Header::buf(bytes));
        Ok(Self {
            header,
            margin: 0,
            max_margin: 1000,
            file,
            file_map,
        })
    }

    pub fn next_entry_id(&self) -> entry::Id<E> {
        self.header.next_entry_id
    }

    fn entry_offset(&self, id: entry::Id<E>) -> usize {
        Header::<E, M>::len() + E::len() * id.as_usize()
    }

    pub fn last_entry_id(&self) -> Option<entry::Id<E>> {
        match self.next_entry_id().as_u64() {
            0 => None,
            _ => Some(self.next_entry_id() - 1),
        }
    }

    fn header_buf(&self) -> entry::BufConst<'_, Header<E, M>> {
        let bytes = entry::bytes::Const::new(self.file_map.get_unchecked(0 .. Header::<E, M>::len()));
        Header::buf(bytes)
    }

    fn header_buf_mut(&self) -> entry::BufMut<'_, Header<E, M>> {
        let bytes = entry::bytes::Mut::new(self.file_map.get_unchecked_mut(0 .. Header::<E, M>::len()));
        Header::buf(bytes)
    }

    fn set_next_entry_id(&self, value: entry::Id<E>) {
        self.header.next_entry_id = value;
        self.next_entry_id().encode(self.header_buf_mut().next_entry_id());
    }

    // region: Core functions.
    pub unsafe fn buf_unchecked(&self, id: entry::Id<E>) -> entry::BufConst<'_, E> {
        let offset = self.entry_offset(id);
        let bytes = entry::Bytes::new(self.file_map.get_unchecked(offset..offset + E::len()));
        E::buf(bytes)
    }

    pub unsafe fn buf_unchecked_mut(&self, id: entry::Id<E>) -> entry::BufMut<'_, E> {
        let offset = self.entry_offset(id);
        let bytes = entry::Bytes::new(self.file_map.get_unchecked_mut(offset..offset + E::len()));
        E::buf(bytes)
    }

    pub fn buf(&self, id: entry::Id<E>) -> Option<entry::BufConst<'_, E>> {
        if self.next_entry_id() > id {
            Some(unsafe { self.buf_unchecked(id) })
        } else {
            None
        }
    }

    pub fn buf_mut(&mut self, id: entry::Id<E>) -> Option<entry::BufMut<'_, E>> {
        if self.next_entry_id() > id {
            Some(unsafe { self.buf_unchecked_mut(id) })
        } else {
            None
        }
    }
    // endregion: Core functions.

    pub fn add(&mut self, entry: &impl entry::Readable<E>) -> Result<entry::Id<E>, AddError> {
        type E = AddError;
        let id = self.next_entry_id();
        if self.margin == 0 {
            let new_len = self.entry_offset(id + self.max_margin + 2);
            self.file.set_len(new_len as u64).map_err(E::Io)?;
            self.file_map = unsafe { MmapOptions::new().len(new_len).map_mut(&self.file).map_err(E::Io)? };
            self.margin = self.max_margin + 1;
        }
        self.margin -= 1;
        entry.write_to(unsafe { self.buf_unchecked_mut(id) });
        self.set_next_entry_id(self.next_entry_id() + 1);
        Ok(id)
    }

    pub fn remove_last(&mut self) -> Result<(), RemoveLastError> {
        type E = RemoveLastError;
        let id = self.next_entry_id();
        if self.margin >= self.max_margin {
            let new_len = self.entry_offset(id);
            self.file.set_len(new_len as u64).map_err(E::Io)?;
            self.file_map = unsafe { MmapOptions::new().len(new_len).map_mut(&self.file).map_err(E::Io)? };
            self.margin = 0;
        }
        self.margin += 1;
        self.set_next_entry_id(id - 1);
        Ok(())
    }

    pub fn all_ids(&self) -> impl Iterator<Item = entry::Id<E>> {
        entry::id::Range(entry::Id::zero(), self.next_entry_id())
    }

    // Convenience functions.
    pub fn find<L: Lens<In = E>>(
        &self,
        lens: L,
        // ids: impl Iterator<Item = entry::Id<Entry>>,
        f: impl Fn(entry::BufConst<'_, L::Out>) -> bool,
    ) -> Result<Option<(entry::Id<E>, entry::BufConst<'_, L::Out>)>, GetError> {
        for id in self.all_ids() {
            let buf = lens.apply(unsafe { self.buf_unchecked(id) });
            if f(L::Out::buf_rb_const(&buf)) {
                return Ok(Some((id, buf)));
            }
        }
        Ok(None)
    }

    // Doesn't check if src_id or dst_id are valid.
    pub unsafe fn copy<L: Lens<In = E> + Clone>(
        &mut self,
        lens: L,
        src_id: entry::Id<E>,
        dst_id: entry::Id<E>,
    ) {
        let mut dst = unsafe { entry::buf_detach(lens.clone().apply(self.buf_unchecked_mut(dst_id))) };
        let src = lens.apply(self.buf_unchecked(src_id));
        <L::Out as Entry>::buf_copy_to(src, dst);
    }

    pub fn swap<L: Lens<In = E> + Clone>(
        &mut self,
        lens: L,
        a_id: entry::Id<E>,
        b_id: entry::Id<E>,
    ) {
        if a_id != b_id {
            let mut a = unsafe { entry::buf_detach(lens.apply(self.buf_unchecked_mut(a_id))) };
            let mut b = lens.apply(unsafe { self.buf_unchecked_mut(b_id) });
            entry::buf_swap(a, b);
        }
    }

    /// Swaps given entry with the last entry and removes it.
    /// WARNING: This method changes ID of the last entry in this collection.
    /// You probably should only use this if this collection is used as a stack/set and not as a map.
    /// Doesn't check if id is valid.
    pub unsafe fn swap_remove(&mut self, id: entry::Id<E>) -> Result<(), SwapRemoveError> {
        type E = SwapRemoveError;
        if let Some(last_entry_id) = self.last_entry_id() {
            self.swap(lens::identity(), id, last_entry_id);
        }
        self.remove_last().map_err(E::RemoveLastError)?;
        Ok(())
    }
}

impl<E: Entry, M: Entry + entry::Codable> Value<E, M> {
    pub fn get<L: Lens<In = E>>(&self, lens: L, id: entry::Id<E>) -> Option<L::Out>
    where
        L::Out: entry::Codable,
    {
        self.buf(id).map(|x| L::Out::decode(lens.apply(x)))
    }

    pub fn set<L: Lens<In = E>>(
        &self,
        lens: L,
        id: entry::Id<E>,
        value: impl entry::Readable<L::Out>,
    ) -> bool
    where
        L::Out: entry::Codable,
    {
        if let Some(mut buf) = self.buf_mut(id) {
            value.write_to(buf);
            true
        } else {
            false
        }
    }

    pub fn linear_search<L: Lens<In = E>>(&self, lens: L, value: impl entry::Readable<L::Out>) {
        let buf = value.into_buf();
    }
}
