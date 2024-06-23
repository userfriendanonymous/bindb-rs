
fn main() {
    let owner1 = Owner1(5);
    let holder1 = Holder1;
    println!("{owner1:?}");
}

#[derive(Debug, Clone, Copy)]
struct Owner1(u32);

impl Owner for Owner1 {
    type H = Holder1;
}

struct Holder1;
impl Holder for Holder1 {
    type O = Owner1;
}

trait Owner {
    type H: Holder;
}

trait Holder {
    type O: Owner;
}

impl Owner1 {
    fn idk<Holder>(p: Holder) {
        
    }
}