use crate::{bytes_ptr, dynamic::{self}, BytesPtr, Dynamic};
use super::{arb_num::{self, Base}, ArbNum};

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

dynamic! {
    pub struct BytesPtrCLL<const LL: usize>(bytes_ptr::Const);
    buf! { pub struct BytesPtrCLLBuf<P, const LL: usize>(BytesPtrCLL<LL>, P); }
    impl<const LL: usize> I for BytesPtrCLL<LL> { type Buf<P> = BytesPtrCLLBuf<P, LL>; }
}

impl<'a, const LL: usize> Dynamic for BytesPtrCLL<LL> {
    fn len(&self) -> usize {
        LL + self.0.len()
    }
    fn buf_len(buf: dynamic::BufConst<Self>) -> usize {
        unsafe { dynamic::decode_ptr::<ArbNum<LL, u64>>(buf.0).0.unwrap() as usize + LL }
    }
    fn encode(&self, buf: dynamic::BufMut<Self>) -> usize {
        unsafe {
            let len = self.0.len();
            dynamic::encode_ptr(buf.0, &ArbNum::<LL, u64>::new(len as u64));
            buf.0.range_at(LL, len).copy_from(self.0);
            LL + len
        }
    }
}

impl<'a, const LL: usize> dynamic::Decode for BytesPtrCLL<LL> {
    fn decode(buf: dynamic::BufConst<Self>) -> (Self, usize) {
        unsafe {
            let len = dynamic::decode_ptr::<ArbNum<LL, u64>>(buf.0).0.unwrap() as usize;
            (Self(buf.0.range_at(LL, len)), LL + len)
        }
    }
}

dynamic! {
    pub struct StringCLL<const LL: usize>(String);
    buf! { pub struct StringCLLBuf<P, const LL: usize>(StringCLL<LL>, P); }
    impl<const LL: usize> I for StringCLL<LL> { type Buf<P> = StringCLLBuf<P, LL>; }
}

impl<const LL: usize> StringCLL<LL> {
    fn try_from_string(value: String) -> Option<Self> {
        if (value.len() as u64).fits_in_bytes(LL) {
            Some(Self(value))
        } else {
            None
        }
    }

    fn from_string(value: String) -> Self {
        Self::try_from_string(value).unwrap()
    }
}

impl<const LL: usize> Dynamic for StringCLL<LL> {
    fn len(&self) -> usize {
        String::len(&self.0)
    }
    fn buf_len(buf: dynamic::BufConst<Self>) -> usize {
        let (len, _) = unsafe { dynamic::decode_ptr::<ArbNum<LL, u64>>(buf.0) };
        len.unwrap() as usize
    }
    fn encode(&self, buf: dynamic::BufMut<Self>) -> usize {
        unsafe { dynamic::encode_ptr(buf.0, &BytesPtrCLL::<LL>(bytes_ptr::Const::from_slice(self.0.as_bytes()))) }
    }
}

impl<const LL: usize> dynamic::Decode for StringCLL<LL> {
    fn decode(buf: dynamic::BufConst<Self>) -> (Self, usize) {
        let (bytes, len) = unsafe { dynamic::decode_ptr::<BytesPtrCLL<LL>>(buf.0) };
        (Self(String::from_utf8_lossy(bytes.0.slice()).into_owned()), len)
    }
}