#![feature(generic_const_exprs)]

use std::marker::PhantomData;


pub struct BytesRef<'a, T>(&'a [u8], PhantomData<T>) where T: ?Sized;

impl<'a, T> BytesRef<'a, T> {
    fn new(data: &'a [u8]) -> Self {
        Self(data, Default::default())
    }

    pub fn to<O: Codable>(&self, at: usize) -> BytesRef<'a, O> {
        BytesRef::new(&self.0[at .. at + O::SIZE])
    }
}

pub trait Codable {
    const SIZE: usize;
    fn encode(&self, bytes: BytesRef<'_, Self>);
}

impl<T: Codable> Codable for Option<T> {
    const SIZE: usize = T::SIZE + 1;

    fn encode(&self, bytes: BytesRef<'_, Self>) {
        match self {
            Some(t) => t.encode(bytes.to(1)),
            None => {}
        }
    }
    // fn encode(&self, bytes: &[u8]) {
    //     match self {
    //         Some(t) => t.encode(&bytes[0 .. ]),
    //         None => {}
    //     }
    // }

}

#[test]
fn test() {

}