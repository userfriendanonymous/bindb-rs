use crate::{Entry, entry::{self}};

entry! {
    #[derive(Clone, Debug)]
    pub struct Value<E, M: Entry> {
        #[lens(pub buf_next_entry_id)]
        pub next_entry_id: entry::Id<E>,
        #[lens(pub buf_meta)]
        pub meta: M
    }

    buf! { pub struct Buf<P, E, M: Entry>(Value<E, M>, P); }

    impl<E, M: Entry> I for Value<E, M> {
        type Buf<P> = Buf<P, E, M>;
    }

    impl<E, M: Entry + entry::Codable> Codable for Value<E, M> {}
}

impl<P: entry::Ptr, E, M: entry::Codable> Buf<P, E, M> {
    pub fn next_entry_id(self) -> entry::Buf<entry::Id<E>, P> {
        Value::buf_next_entry_id(self)
    }
}