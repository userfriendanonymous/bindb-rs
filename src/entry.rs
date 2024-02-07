use std::{hash::Hash, marker::PhantomData, ops::{Add, Sub}};



// #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<T>(u64, PhantomData<T>);

impl<T> From<u64> for Id<T> {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T> Id<T> {
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

    pub fn next(&mut self) -> Self {
        let value = self.clone();
        self.0 += 1;
        value
    }
}

impl<T> Add<u64> for Id<T> {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self::new(self.as_u64() + rhs)
    }
}

impl<T> Sub<u64> for Id<T> {
    type Output = Self;
    
    fn sub(self, rhs: u64) -> Self::Output {
        Self::new(self.as_u64() - rhs)
    }
}
