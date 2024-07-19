use std::{fs::File, marker::PhantomData};
use binbuf::{BytesPtr, bytes_ptr};
use memmap2::MmapMut;

#[derive(Debug)]
pub enum OpenError {
    Io(std::io::Error)
}

#[derive(Debug)]
pub enum CreateError {
    Io(std::io::Error)
}

#[derive(Debug)]
pub enum SetError {
    Io(std::io::Error),
}

pub struct Value<T> {
    file: File,
    mmap: MmapMut,
    len: usize,
    _marker: PhantomData<fn() -> T>
}

impl<T: binbuf::Dynamic> Value<T> {
    pub unsafe fn open(file: File) -> Result<Self, OpenError> {
        let mmap = MmapMut::map_mut(&file).map_err(OpenError::Io)?;
        let len = binbuf::dynamic::ptr_len::<T>(bytes_ptr::Const::from_slice(&mmap[0 .. ]));
        Ok(Self { file, mmap, len, _marker: PhantomData })
    }

    pub unsafe fn create(file: File, value: impl binbuf::dynamic::Readable<T>) -> Result<Self, CreateError> {
        let len = value.len();
        file.set_len(len as u64).map_err(CreateError::Io)?;
        let mut mmap = MmapMut::map_mut(&file).map_err(CreateError::Io)?;
        let buf = unsafe { T::buf(bytes_ptr::Mut::from_slice(&mut mmap[0 .. ])) };
        value.write_to(buf);
        Ok(Self { file, mmap, len, _marker: PhantomData })
    }

    pub fn set(&mut self, value: impl binbuf::dynamic::Readable<T>) -> Result<usize, SetError> {
        let len = value.len();
        if len > self.len {
            self.file.set_len(len as u64).map_err(SetError::Io)?;
            self.mmap = unsafe { MmapMut::map_mut(&self.file) }.map_err(SetError::Io)?;
            self.len = len;
        }
        let written_len = value.write_to(self.buf_mut());
        debug_assert_eq!(len, written_len);
        Ok(len)
    }

    pub fn buf(&self) -> binbuf::BufConst<T> {
        unsafe { T::buf(bytes_ptr::Const::from_slice(&self.mmap[0 .. ])) }
    }

    pub fn buf_mut(&mut self) -> binbuf::BufMut<T> {
        unsafe { T::buf(bytes_ptr::Mut::from_slice(&mut self.mmap[0 .. ])) }
    }
}

impl<T: binbuf::dynamic::Decode> Value<T> {
    pub fn get(&self) -> T {
        T::decode(self.buf()).0
    }
}