use std::{fs::File, marker::PhantomData, path::{Path, PathBuf}, pin::pin};
use binbuf::{bytes_ptr, fixed::BufPartialEq, BytesPtr, Entry, Fixed as _};
use super::OpenMode;

pub use {entry_id::Value as EntryId, header::Value as Header};
use memmap2::{Mmap, MmapMut, MmapOptions};

pub mod entry_id;
pub mod header;

binbuf::fixed! {
    #[derive(Clone)]
    pub struct FreeLocation {
        #[lens(buf_start)]
        start: u64,
        #[lens(buf_end)]
        end: u64,
    }

    buf! { pub struct FreeLocationBuf<P>(FreeLocation, P); }
    impl I for FreeLocation {
        type Buf<P> = FreeLocationBuf<P>;
    }
    impl Code for FreeLocation {}
}

impl<P: BytesPtr> FreeLocationBuf<P> {
    fn start(self) -> binbuf::Buf<u64, P> { FreeLocation::buf_start(self) }
    fn end(self) -> binbuf::Buf<u64, P> { FreeLocation::buf_end(self) }
}

#[derive(Debug)]
pub enum AddError {
    Io(std::io::Error),
    FixedSwapRemove(super::fixed::SwapRemoveError),
}

#[derive(Debug)]
pub enum RemoveError {
    Io(std::io::Error),
    FixedSwapRemove(super::fixed::SwapRemoveError),
    FixedAdd(super::fixed::AddError),
}

#[derive(Debug)]
pub enum OpenError {
    Io(std::io::Error),
    FixedOpen(super::fixed::OpenError)
}

pub struct OpenFiles {
    pub entries: File,
    pub free_locations: File,
}

pub struct OpenMaxMargins {
    pub entries: u64,
    pub free_locations: u64,
}


pub struct OpenConfig {
    pub mode: OpenMode,
    pub files: OpenFiles,
    pub max_margins: OpenMaxMargins,
}

pub struct Value<E> {
    len: u64,
    bytes_len: u64,
    free_locations: super::Fixed<FreeLocation>,
    entries_file: File,
    entries_mmap: MmapMut,
    margin: u64,
    max_margin: u64,
    _marker: PhantomData<fn() -> E>
}

impl<E: binbuf::Dynamic> Value<E> {
    pub unsafe fn open(OpenConfig { mode, files, max_margins }: OpenConfig) -> Result<Self, OpenError> {
        if let OpenMode::New = mode {
            files.entries.set_len(Header::LEN as u64).map_err(OpenError::Io)?;
        }
        let mut entries_mmap = MmapMut::map_mut(&files.entries).map_err(OpenError::Io)?;
        let header = match mode {
            OpenMode::Existing => binbuf::fixed::decode::<Header, _>(
                Header::buf(bytes_ptr::Const::new(entries_mmap[0 .. Header::LEN].as_ptr(), Header::LEN))
            ),
            OpenMode::New => unsafe {
                binbuf::fixed::encode_ptr(
                    bytes_ptr::Mut::from_slice(&mut entries_mmap[0 .. Header::LEN]),
                    &Header { len: 0, bytes_len: 0 }
                );
                Header { len: 0, bytes_len: 0 }
            }
        };
        Ok(Self {
            len: header.len,
            bytes_len: header.bytes_len,
            free_locations: super::Fixed::open(mode, files.free_locations, max_margins.free_locations).map_err(OpenError::FixedOpen)?,
            entries_file: files.entries,
            entries_mmap,
            margin: 0,
            max_margin: max_margins.entries,
            _marker: PhantomData
        })
    }

    fn entry_offset(&self, id: EntryId) -> usize {
        Header::LEN + id.0 as usize
    }

    fn header_buf(&self) -> binbuf::BufConst<Header> {
        let ptr = unsafe { bytes_ptr::Const::from_slice(&self.entries_mmap[0 .. Header::LEN]) };
        unsafe { Header::buf(ptr) }
    }

    fn header_buf_mut(&mut self) -> binbuf::BufMut<Header> {
        let ptr = unsafe { bytes_ptr::Mut::from_slice(&mut self.entries_mmap[0 .. Header::LEN]) };
        unsafe { Header::buf(ptr) }
    }

    fn set_bytes_len(&mut self, value: u64) {
        self.bytes_len = value;
        value.encode(Header::buf_bytes_len(self.header_buf_mut()));
    }

    // Doesn't check if id is valid. It's impossible to check that.
    // Id may be pointing to garbage.
    pub unsafe fn buf_unchecked(&self, id: EntryId) -> binbuf::BufConst<E> {
        let ptr = bytes_ptr::Const::from_slice(
            self.entries_mmap.get_unchecked(self.entry_offset(id) ..)
        );
        E::buf(ptr)
    }

    pub unsafe fn buf_mut_unchecked(&mut self, id: EntryId) -> binbuf::BufMut<E> {
        let offset = self.entry_offset(id);
        let ptr = bytes_ptr::Mut::from_slice(
            self.entries_mmap.get_unchecked_mut(offset ..)
        );
        E::buf(ptr)
    }

    pub fn add(&mut self, entry: impl binbuf::dynamic::Readable<E>) -> Result<EntryId, AddError> {
        let entry_len = entry.len();
        let entry_len_u64 = entry_len as u64;
        for loc_id in self.free_locations.all_ids() {
            let (loc, _) = binbuf::dynamic::decode::<FreeLocation>(unsafe { self.free_locations.buf_unchecked(loc_id) });
            let loc_len = loc.end - loc.start;
            if loc_len >= entry_len_u64 {
                let entry_id = EntryId(loc.start);
                let buf = unsafe { self.buf_mut_unchecked(entry_id) };
                let written_len = entry.write_to(buf);
                debug_assert_eq!(written_len, entry_len);
                let left_len = loc_len - entry_len_u64;
                if left_len == 0 {
                    unsafe { self.free_locations.swap_remove(loc_id).map_err(AddError::FixedSwapRemove) }?;
                } else {
                    self.free_locations.set(loc_id, &FreeLocation {
                        start: loc.start + entry_len_u64,
                        end: loc.end,
                    });
                }
                return Ok(entry_id);
            }
        }

        if self.margin < entry_len_u64 {
            let margin_extra = self.margin + ((entry_len_u64 - self.margin) / self.max_margin + 1) * self.max_margin;
            let new_len = self.entry_offset(
                EntryId(self.bytes_len + margin_extra)
            );
            self.entries_file.set_len(new_len as u64).map_err(AddError::Io)?;
            self.entries_mmap = unsafe { MmapOptions::new().len(new_len).map_mut(&self.entries_file).map_err(AddError::Io)? };

            let entry_id = EntryId(self.bytes_len);
            let written_len = entry.write_to(unsafe { self.buf_mut_unchecked(entry_id) });
            debug_assert_eq!(written_len, entry_len);
            self.set_bytes_len(self.bytes_len + entry_len_u64);
            self.margin = margin_extra - entry_len_u64;
            Ok(entry_id)

        } else {
            let entry_id = EntryId(self.bytes_len);
            let written_len = entry.write_to(unsafe { self.buf_mut_unchecked(entry_id) });
            debug_assert_eq!(written_len, entry_len);
            self.set_bytes_len(self.bytes_len + entry_len_u64);
            self.margin -= entry_len_u64;
            Ok(entry_id)
        }
    }

    pub fn free_locations_len(&self) -> u64 {
        self.free_locations.len()
    }

    pub unsafe fn remove(&mut self, id: EntryId) -> Result<(), RemoveError> {
        let entry_len = binbuf::dynamic::buf_len::<E>(self.buf_unchecked(id));
        let entry_len_u64 = entry_len as u64;
        let mut entry_loc_store = pin!([0; FreeLocation::LEN]);
        let entry_loc = FreeLocation { start: id.0, end: id.0 + entry_len_u64 };

        let entry_loc_buf = binbuf::entry::buf_mut_from_slice::<FreeLocation>(&mut *entry_loc_store);
        entry_loc.encode(entry_loc_buf);

        let is_last_entry = entry_loc.end == self.bytes_len;

        let mut loc_expanded = (false, is_last_entry);
        let mut loc_id = 0u64;
        
        loop {
            if loc_id >= self.free_locations.len() {
                if !loc_expanded.0 && !loc_expanded.1 {
                }
                break;
            }
            let loc_buf = self.free_locations.buf_unchecked(loc_id);

            if !loc_expanded.0 && binbuf::fixed::decode::<u64, _>(loc_buf.end()).buf_eq(
                binbuf::fixed::buf_to_const::<u64, _>(entry_loc_buf.start())
            ) {
                binbuf::fixed::buf_copy_to::<u64>(loc_buf.start(), entry_loc_buf.start());
                self.free_locations.swap_remove(loc_id).map_err(RemoveError::FixedSwapRemove)?;
                loc_expanded.0 = true;

            } else if !loc_expanded.1 && binbuf::fixed::decode::<u64, _>(loc_buf.start()).buf_eq(
                binbuf::fixed::buf_to_const::<u64, _>(entry_loc_buf.end())
            ) {
                binbuf::fixed::buf_copy_to::<u64>(loc_buf.end(), entry_loc_buf.end());
                self.free_locations.swap_remove(loc_id).map_err(RemoveError::FixedSwapRemove)?;
                loc_expanded.1 = true;

            } else {
                loc_id += 1;
            }

            if loc_expanded.0 && loc_expanded.1 {
                break;
            }
        }

        if is_last_entry {
            let size_dec = binbuf::fixed::decode::<u64, _>(entry_loc_buf.end())
                - binbuf::fixed::decode::<u64, _>(entry_loc_buf.start());
            self.set_bytes_len(self.bytes_len - size_dec);
            self.margin += size_dec;
            if self.margin >= self.max_margin {
                let new_len = self.entry_offset(EntryId(self.bytes_len + self.margin % self.max_margin));
                self.entries_file.set_len(new_len as u64).map_err(RemoveError::Io)?;
                self.entries_mmap = unsafe { MmapOptions::new().len(new_len).map_mut(&self.entries_file).map_err(RemoveError::Io)? };
                self.margin = self.margin % self.max_margin;
            }
        } else {
            self.free_locations.add(entry_loc_buf).map_err(RemoveError::FixedAdd)?;
        }
        drop(entry_loc_store);
        Ok(())
    }
}

impl<E: binbuf::dynamic::Decode> Value<E> {
    // Make sure ID is valid!
    pub unsafe fn get(&self, id: EntryId) -> E {
        E::decode(self.buf_unchecked(id)).0
    }
}