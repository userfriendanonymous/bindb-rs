
binbuf::fixed! {
    #[derive(Clone, Copy)]
    pub struct Value(pub u64);

    buf! { pub struct Buf<P>(Value, P); }
    impl I for Value {
        type Buf<P> = Buf<P>;
    }
    impl Code for Value {}
}
