use binbuf::{BytesPtr, bytes_ptr, Fixed as _, Entry as _};
use crate::utils::{slice_to_array, slice_to_array_mut};
use std::{fs::File, marker::PhantomData, path::Path};
use memmap2::{MmapAsRawDesc, MmapMut, MmapOptions};
pub use header::Value as Header;
use super::OpenMode;

pub mod header;

#[derive(Debug)]
pub enum GetError {
    InvalidId,
}

#[derive(Debug)]
pub enum AddError {
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum RemoveLastError {
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum SwapRemoveError {
    RemoveLastError(RemoveLastError),
    //Fmmap(fmmap::error::Error),
}

#[derive(Debug)]
pub enum CreateError {
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum OpenError {
    Io(std::io::Error),
}

pub struct OpenConfig {
    pub mode: OpenMode,
    pub file: File,
    pub max_margin: u64,
}

pub struct Value<E> {
    next_entry_id: u64,
    file: File,
    file_map: MmapMut,
    margin: u64,
    max_margin: u64,
    _marker: PhantomData<fn() -> E>
}

impl<E: binbuf::Fixed> Value<E> {
    pub unsafe fn open(mode: OpenMode, file: File, max_margin: u64) -> Result<Self, OpenError> {
        let header_len = Header::LEN;
        if let OpenMode::New = &mode {
            file.set_len(header_len as u64).map_err(OpenError::Io)?;
        }
        let mut file_map = MmapMut::map_mut(&file).map_err(OpenError::Io)?;
        let ptr = bytes_ptr::Const::new(file_map[0 .. header_len].as_ptr(), header_len);
        let next_entry_id = match mode {
            OpenMode::Existing => binbuf::fixed::decode(Header::buf(ptr).next_entry_id()),
            OpenMode::New => unsafe {
                binbuf::fixed::encode_ptr(
                    bytes_ptr::Mut::from_slice(&mut file_map[0 .. header_len]),
                    &Header { next_entry_id: 0 }
                );
                0
            }
        };
        Ok(Self {
            next_entry_id,
            margin: 0,
            max_margin,
            file,
            file_map,
            _marker: PhantomData
        })
    }

    pub fn len(&self) -> u64 {
        self.next_entry_id
    }

    pub fn last_id(&self) -> u64 {
        self.len() - 1
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // pub fn next_entry_id(&self) -> u64 {
    //     self.next_entry_id
    // }

    fn entry_offset(&self, id: u64) -> usize {
        Header::LEN + E::LEN * (id as usize)
    }

    // pub fn meta_buf(&self) -> binbuf::BufConst<E> {
    //     self.header_buf().meta()
    // }

    // pub fn meta_buf_mut(&mut self) -> binbuf::BufMut<E> {
    //     self.header_buf_mut().meta()
    // }

    pub fn last_entry_id(&self) -> Option<u64> {
        match self.next_entry_id {
            0 => None,
            _ => Some(self.next_entry_id - 1),
        }
    }

    fn header_buf(&self) -> binbuf::BufConst<Header> {
        let len = Header::LEN;
        let ptr = unsafe { bytes_ptr::Const::from_slice(self.file_map.get_unchecked(0 .. len)) };
        unsafe { Header::buf(ptr) }
    }

    fn header_buf_mut(&mut self) -> binbuf::BufMut<Header> {
        let len = Header::LEN;
        let ptr = unsafe { bytes_ptr::Mut::from_slice(self.file_map.get_unchecked_mut(0 .. len)) };
        unsafe { Header::buf(ptr) }
    }

    fn set_next_entry_id(&mut self, value: u64) {
        self.next_entry_id = value;
        let v = self.next_entry_id;
        v.encode(self.header_buf_mut().next_entry_id());
    }

    // region: Core functions.
    // Doesn't check if ID is valid.
    pub unsafe fn buf_unchecked(&self, id: u64) -> binbuf::BufConst<E> {
        let offset = self.entry_offset(id);
        let ptr = bytes_ptr::Const::new(self.file_map.get_unchecked(offset .. offset + E::LEN).as_ptr(), E::LEN);
        E::buf(ptr)
    }

    pub unsafe fn buf_mut_unchecked(&mut self, id: u64) -> binbuf::BufMut<E> {
        let offset = self.entry_offset(id);
        let ptr = bytes_ptr::Mut::new(self.file_map.get_unchecked_mut(offset .. offset + E::LEN).as_mut_ptr(), E::LEN);
        E::buf(ptr)
    }

    pub fn last_buf(&self) -> Option<binbuf::BufConst<E>> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.buf_unchecked(self.len() - 1) })
        }
    }

    pub fn is_id_valid(&self, id: u64) -> bool {
        self.next_entry_id > id
    }

    pub fn buf(&self, id: u64) -> binbuf::BufConst<E> {
        if self.is_id_valid(id) {
            unsafe { self.buf_unchecked(id) }
        } else {
            panic!("Id is invalid: {id}")
        }
    }

    pub fn buf_mut(&mut self, id: u64) -> binbuf::BufMut<E> {
        if self.is_id_valid(id) {
            unsafe { self.buf_mut_unchecked(id) }
        } else {
            panic!("Id is invalid: {id}")
        }
    }
    // endregion: Core functions.

    pub fn add(&mut self, entry: impl binbuf::fixed::Readable<E>) -> Result<u64, AddError> {
        let id = self.next_entry_id;
        if self.margin == 0 {
            let new_len = self.entry_offset(id + self.max_margin + 2);
            self.file.set_len(new_len as u64).map_err(AddError::Io)?;
            self.file_map = unsafe { MmapOptions::new().len(new_len).map_mut(&self.file).map_err(AddError::Io)? };
            self.margin = self.max_margin + 1;
        }
        self.margin -= 1;
        entry.write_to(unsafe { self.buf_mut_unchecked(id) });
        self.set_next_entry_id(self.next_entry_id + 1);
        Ok(id)
    }

    pub fn remove_last(&mut self) -> Result<(), RemoveLastError> {
        let id = self.next_entry_id;
        if self.margin >= self.max_margin {
            let new_len = self.entry_offset(id);
            self.file.set_len(new_len as u64).map_err(RemoveLastError::Io)?;
            self.file_map = unsafe { MmapOptions::new().len(new_len).map_mut(&self.file).map_err(RemoveLastError::Io)? };
            self.margin = 0;
        }
        self.margin += 1;
        self.set_next_entry_id(id - 1);
        Ok(())
    }

    // Removes if ID is last.
    // Returns Ok(true) if ID is NOT last, and Ok(false) if ID is last and is successfully removed.
    pub fn remove_if_last(&mut self, id: u64) -> Result<bool, RemoveLastError> {
        if id == self.last_id() {
            self.remove_last()?;
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn all_ids(&self) -> impl Iterator<Item = u64> {
        0 .. self.next_entry_id
    }

    // Convenience functions.
    // pub fn find<Out: Entry>(
    //     &self,
    //     lens: impl Lens<E, Out> + Clone,
    //     // ids: impl Iterator<Item = entry::Id<Entry>>,
    //     f: impl Fn(entry::BufConst<Out>) -> bool,
    // ) -> Result<Option<(u64, entry::BufConst<Out>)>, GetError> {
    //     for id in self.all_ids() {
    //         let buf = lens.clone().apply(unsafe { self.buf_unchecked(id) });
    //         if f(buf) {
    //             return Ok(Some((id, buf)));
    //         }
    //     }
    //     Ok(None)
    // }

    // Doesn't check if src_id or dst_id are valid.
    pub unsafe fn copy(
        &mut self,
        src_id: u64,
        dst_id: u64,
    ) {
        let mut dst = self.buf_mut_unchecked(dst_id);
        let src = self.buf_unchecked(src_id);
        binbuf::fixed::buf_copy_to::<E>(src, dst);
    }

    // Doesn't check if a_id or b_id are valid.
    // It's OK if a_id == b_id.
    pub unsafe fn swap(
        &mut self,
        a_id: u64,
        b_id: u64,
    ) {
        if a_id != b_id {
            let mut a = self.buf_mut_unchecked(a_id);
            let mut b = self.buf_mut_unchecked(b_id);
            binbuf::fixed::buf_swap::<E>(a, b);
        }
    }

    /// Swaps given entry with the last entry and removes it.
    /// WARNING: This method changes ID of the last entry in this collection.
    /// You probably should only use this if this collection is used as a stack/set and not as a map.
    /// Doesn't check if id is valid.
    pub unsafe fn swap_remove(&mut self, id: u64) -> Result<(), SwapRemoveError> {
        if let Some(last_entry_id) = self.last_entry_id() {
            self.swap(id, last_entry_id);
        }
        self.remove_last().map_err(SwapRemoveError::RemoveLastError)?;
        Ok(())
    }

    pub fn set(&mut self, id: u64, value: impl binbuf::fixed::Readable<E>) {
        if self.is_id_valid(id) {
            unsafe { value.write_to(self.buf_mut_unchecked(id)) }
        } else {
            panic!("Invalid id: {id}");
        }
    }
}

impl<E: binbuf::fixed::Decode> Value<E> {
    pub fn get(&self, id: u64) -> E {
        E::decode(self.buf(id))
    }
}