use std::{fmt::Debug, hash::Hash, marker::PhantomData, ops::{Add, Sub}};

// #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
entry! {
    pub struct Value(u64);
    buf! { pub struct Buf<P, T>(Value<T>, P); }

    impl<T> I for Value<T> {
        type Buf<P> = Buf<P, T>;
    }
    impl<T> Codable for Value<T> {}
}

impl<T> Debug for Value<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Id({})", self.0))
    }
}

impl<T> From<u64> for Value<T> {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl<T> Clone for Value<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl<T> Copy for Value<T> {}

impl<T> PartialEq for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Value<T> {}

impl<T> PartialOrd for Value<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Value<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Hash for Value<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T> Value<T> {
    fn new(n: u64) -> Self {
        Self(n, Default::default())
    }

    pub fn from_u64(n: u64) -> Self {
        Self::new(n)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn zero() -> Self {
        Self::new(0)
    }

    pub fn succ(&self) -> Self {
        *self + 1
    }

    // pub fn next(&mut self) -> Self {
    //     let value = self.clone();
    //     self.0 += 1;
    //     value
    // }

    // pub fn prev(&self) -> Self {
    //     let value = self.clone();
    //     self.0 -= 1;
    //     value
    // }
}

impl<T> Add<u64> for Value<T> {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self::new(self.as_u64() + rhs)
    }
}

impl<T> Sub<u64> for Value<T> {
    type Output = Self;
    
    fn sub(self, rhs: u64) -> Self::Output {
        Self::new(self.as_u64() - rhs)
    }
}

pub struct Range<T>(pub Value<T>, pub Value<T>);

impl<T> Iterator for Range<T> {
    type Item = Value<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 >= self.1 {
            None
        } else {
            let value = self.0.clone();
            self.0.0 += 1;
            Some(value)
        }
    }
}