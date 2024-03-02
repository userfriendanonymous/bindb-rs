pub trait Instance<In> {
    type Out;
}

pub type Apply<In, F: Instance<In>> = F::Out;
