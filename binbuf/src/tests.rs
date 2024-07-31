

dynamic! {
    pub struct Wow {
        #[lens(buf_idk)]
        idk: u64,
    }
    buf! { pub struct WowBuf<P>(Wow, P); }

    impl I for Wow {
        type Buf<P> = WowBuf<P>;
    }

    impl Code for Wow {}
}