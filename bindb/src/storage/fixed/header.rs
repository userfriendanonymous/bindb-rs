use binbuf::BytesPtr;
use std::marker::PhantomData;

binbuf::fixed! {
    #[derive(Clone, Debug)]
    pub struct Value {
        #[lens(pub buf_next_entry_id)]
        pub next_entry_id: u64,
    }

    buf! { pub struct Buf<P>(Value, P); }

    impl I for Value {
        type Buf<P> = Buf<P>;
    }

    impl Code for Value {}
}

impl<P: BytesPtr> Buf<P> {
    pub fn next_entry_id(self) -> binbuf::Buf<u64, P> {
        Value::buf_next_entry_id(self)
    }
}

// impl<E, M> Value<E, M> {
//     pub fn new(next_entry_id: u64, meta: M) -> Self {
//         Self { next_entry_id, meta, _marker: PhantomData }
//     }
// }