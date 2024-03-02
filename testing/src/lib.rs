#![feature(generic_const_exprs)]
#![feature(trait_alias)]

trait Idk {
    type T<B: Clone + Copy>;
}

fn main() {
    let idk: Box<dyn Idk> = todo!();
}
