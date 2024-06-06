use crate::{Entry, entry::{self}};

entry! {
    #[derive(Clone, Debug)]
    pub struct Value<E, M: Entry> {
        #[lens(pub buf_next_entry_id)]
        pub next_entry_id: entry::Id<E>,
        #[lens(pub buf_meta)]
        pub meta: M
    }

    buf! { struct Buf<BV, E, M>(Value<E, M>, BV); }

    impl<E, M: Entry + entry::Codable> I for Value<E, M> {
        type Buf<BV> = Buf<BV, E, M>;
    }

    impl<E, M: Entry + entry::Codable> Codable for Value<E, M> {}
}

impl<BV: entry::bytes::Variant, E, M: entry::Codable> Buf<BV, E, M> {
    pub fn next_entry_id(self) -> entry::Buf<entry::Id<E>, BV> {
        Value::buf_next_entry_id(self)
    }
}