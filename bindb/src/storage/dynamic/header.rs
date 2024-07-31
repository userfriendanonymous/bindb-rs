use std::marker::PhantomData;

binbuf::fixed! {
    #[derive(Clone, Debug)]
    pub struct Value {
        #[lens(pub buf_len)]
        pub len: u64, // count of items in collection
        #[lens(pub buf_bytes_len)]
        pub bytes_len: u64, // how many bytes taken by storing items in collection (only items, not header)
    }

    buf! { pub struct Buf<P>(Value, P); }

    impl I for Value {
        type Buf<P> = Buf<P>;
    }

    impl Encode for Value {}
    impl Decode for Value {}
}

impl Value {
    pub fn new(len: u64, bytes_len: u64) -> Self {
        Self { len, bytes_len }
    }
}