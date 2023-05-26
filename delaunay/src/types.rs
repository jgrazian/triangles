use std::ops::{Deref, Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertIndex(usize);

impl Default for VertIndex {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl From<usize> for VertIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Deref for VertIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Index<VertIndex> for Vec<T> {
    type Output = T;

    fn index(&self, index: VertIndex) -> &Self::Output {
        &self[index.0]
    }
}

impl<T> IndexMut<VertIndex> for Vec<T> {
    fn index_mut(&mut self, index: VertIndex) -> &mut Self::Output {
        &mut self[index.0]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeIndex(usize);

impl Default for EdgeIndex {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl From<usize> for EdgeIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Deref for EdgeIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Index<EdgeIndex> for Vec<T> {
    type Output = T;

    fn index(&self, index: EdgeIndex) -> &Self::Output {
        &self[index.0]
    }
}

impl<T> IndexMut<EdgeIndex> for Vec<T> {
    fn index_mut(&mut self, index: EdgeIndex) -> &mut Self::Output {
        &mut self[index.0]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertTriple {
    a: VertIndex,
    b: VertIndex,
    c: VertIndex,
}

impl VertTriple {
    pub fn new(a: VertIndex, b: VertIndex, c: VertIndex) -> Self {
        Self { a, b, c }
    }

    pub fn abc(&self) -> (VertIndex, VertIndex, VertIndex) {
        (self.a, self.b, self.c)
    }

    pub fn a(&self) -> VertIndex {
        self.a
    }

    pub fn b(&self) -> VertIndex {
        self.b
    }

    pub fn c(&self) -> VertIndex {
        self.c
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriTriple {
    a: Option<EdgeIndex>,
    b: Option<EdgeIndex>,
    c: Option<EdgeIndex>,
}
impl TriTriple {
    pub const NONE: Self = Self {
        a: None,
        b: None,
        c: None,
    };

    pub fn new(a: Option<EdgeIndex>, b: Option<EdgeIndex>, c: Option<EdgeIndex>) -> Self {
        Self { a, b, c }
    }

    pub fn a(&self) -> Option<EdgeIndex> {
        self.a
    }

    pub fn b(&self) -> Option<EdgeIndex> {
        self.b
    }

    pub fn c(&self) -> Option<EdgeIndex> {
        self.c
    }
}
