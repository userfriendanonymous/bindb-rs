use crate::{bytes_ptr, dynamic::{self}, BytesPtr, Dynamic};

dynamic! {
    buf! { pub struct SliceU8Buf<'a, P>(&'a [u8], P); }
    impl<'a> I for &'a [u8] {
        type Buf<P> = SliceU8Buf<'a, P>;
    }
}

impl<'a> Dynamic for &'a [u8] {
    fn len(&self) -> usize {
        8 + (*self).len()
    }
    fn buf_len(buf: dynamic::BufConst<Self>) -> usize {
        unsafe { dynamic::decode_ptr::<u64>(buf.0).0 as usize + 8 }
    }
    fn encode(&self, buf: dynamic::BufMut<Self>) -> usize {
        unsafe {
            let len = (*self).len();
            dynamic::encode_ptr(buf.0, &(len as u64));
            buf.0.range_at(8, len).copy_from_slice(self);
            8 + len
        }
    }
}

impl<'a> dynamic::Decode for &'a [u8] {
    fn decode(buf: dynamic::BufConst<Self>) -> (Self, usize) {
        unsafe {
            let len = dynamic::decode_ptr::<u64>(buf.0).0 as usize;
            (buf.0.range_at(8, len).slice(), 8 + len)
        }
    }
}

// region: bytes_ptr
dynamic! {
    buf! { pub struct BytesPtrConstBuf<P>(bytes_ptr::Const, P); }
    impl I for bytes_ptr::Const {
        type Buf<P> = BytesPtrConstBuf<P>;
    }
}

impl Dynamic for bytes_ptr::Const {
    fn len(&self) -> usize {
        8 + BytesPtr::len(*self)
    }
    fn buf_len(buf: dynamic::BufConst<Self>) -> usize {
        unsafe { dynamic::decode_ptr::<u64>(buf.0).0 as usize + 8 }
    }
    fn encode(&self, buf: dynamic::BufMut<Self>) -> usize {
        unsafe {
            let len = BytesPtr::len(*self);
            dynamic::encode_ptr(buf.0, &(len as u64));
            buf.0.range_at(8, len).copy_from(*self);
            8 + len
        }
    }
}

impl dynamic::Decode for bytes_ptr::Const {
    fn decode(buf: dynamic::BufConst<Self>) -> (Self, usize) {
        unsafe {
            let len = dynamic::decode_ptr::<u64>(buf.0).0 as usize;
            (buf.0.range_at(8, len), 8 + len)
        }
    }
}


dynamic! {
    buf! { pub struct BytesPtrMutBuf<P>(bytes_ptr::Mut, P); }
    impl I for bytes_ptr::Mut {
        type Buf<P> = BytesPtrMutBuf<P>;
    }
}

impl Dynamic for bytes_ptr::Mut {
    fn len(&self) -> usize {
        8 + BytesPtr::len(*self)
    }
    fn buf_len(buf: dynamic::BufConst<Self>) -> usize {
        unsafe { dynamic::decode_ptr::<u64>(buf.0).0 as usize + 8 }
    }
    fn encode(&self, buf: dynamic::BufMut<Self>) -> usize {
        unsafe {
            let len = BytesPtr::len(*self);
            dynamic::encode_ptr(buf.0, &(len as u64));
            buf.0.range_at(8, len).copy_from(self.to_const());
            8 + len
        }
    }
}
// endregion: bytes_ptr

dynamic! {
    buf! { pub struct StringBuf<P>(String, P); }
    impl I for String {
        type Buf<P> = StringBuf<P>;
    }
}

impl Dynamic for String {
    fn len(&self) -> usize {
        8 + self.len()
    }
    fn buf_len(buf: dynamic::BufConst<Self>) -> usize {
        unsafe { dynamic::ptr_len::<&[u8]>(buf.0) }
    }
    fn encode(&self, buf: dynamic::BufMut<Self>) -> usize {
        unsafe { dynamic::encode_ptr(buf.0, &self.as_bytes()) }
    }
}

impl dynamic::Decode for String {
    fn decode(buf: dynamic::BufConst<Self>) -> (Self, usize) {
        let (bytes, len) = unsafe { dynamic::decode_ptr::<&[u8]>(buf.0) };
        (String::from_utf8_lossy(bytes).into_owned(), len)
    }
}