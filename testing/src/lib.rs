#![feature(generic_const_exprs)]
#![feature(trait_alias)]

fn main() {
    let wow: <Idk<u32> as So>::Hm = todo!();
}

struct Idk<T>(T);

trait So {
    type Hm;
}

impl So for Idk<u32> {
    type Hm = u64;
}

impl So for Idk<String> {
    type Hm = [u8; 5];
}