use super::Codable;

pub trait Instance<Lens> {
    type To: Codable;
    fn offset(lens: &Lens) -> usize;
}

// struct MyItem(u32, Part);
// struct Part(u32);
// struct PartLens;

// impl Lensable<PartLens> for MyItem {
//     type To = Part;
//     fn offset(lens: &PartLens) -> usize {
//         4
//     }
// }