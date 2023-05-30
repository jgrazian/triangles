use std::ops::{Add, Deref, Div, Index, IndexMut, Mul, Sub};

/// A vertex in 2D space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    x: f64,
    y: f64,
}

impl Vertex {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const NAN: Self = Self {
        x: f64::NAN,
        y: f64::NAN,
    };

    pub const INFINITY: Self = Self {
        x: f64::INFINITY,
        y: f64::INFINITY,
    };

    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub const fn splat(value: f64) -> Self {
        Self { x: value, y: value }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    pub fn distance_squared(&self, other: Vertex) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;

        dx * dx + dy * dy
    }

    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn min(&self, other: Vertex) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    pub fn max(&self, other: Vertex) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<(f64, f64)> for Vertex {
    fn from(value: (f64, f64)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<[f64; 2]> for Vertex {
    fn from(value: [f64; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

impl From<Vertex> for (f64, f64) {
    fn from(value: Vertex) -> Self {
        (value.x, value.y)
    }
}

impl From<Vertex> for [f64; 2] {
    fn from(value: Vertex) -> Self {
        [value.x, value.y]
    }
}

impl Add for Vertex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vertex {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f64> for Vertex {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Self::new(self.x * other, self.y * other)
    }
}

impl Mul<Vertex> for f64 {
    type Output = Vertex;

    fn mul(self, other: Vertex) -> Vertex {
        Vertex::new(self * other.x, self * other.y)
    }
}

impl Div<f64> for Vertex {
    type Output = Self;

    fn div(self, other: f64) -> Self {
        Self::new(self.x / other, self.y / other)
    }
}

impl Div<Vertex> for f64 {
    type Output = Vertex;

    fn div(self, other: Vertex) -> Vertex {
        Vertex::new(self / other.x, self / other.y)
    }
}

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
