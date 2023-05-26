use glam::DVec2;

mod types;
mod util;

use types::*;
use util::*;

pub fn triangulate(points: Vec<DVec2>) -> (Triangulation, HullContext) {
    let mut triangulation = Triangulation::new(points);
    let mut hull = triangulation.context();
    triangulation.update_with(&mut hull);
    (triangulation, hull)
}

pub struct Edge<'t> {
    p0: &'t DVec2,
    p1: &'t DVec2,
}

#[derive(Debug)]
pub struct Triangulation {
    points: Vec<DVec2>,
    triangles: Vec<VertIndex>,
    half_edges: Vec<Option<EdgeIndex>>,
    hull: Vec<VertIndex>,
}

/// Port of https://github.com/mapbox/delaunator/blob/main/index.js
impl Triangulation {
    fn new(points: Vec<DVec2>) -> Self {
        let n = points.len();
        let max_triangles = (2 * n - 5).max(0);

        Self {
            points,
            triangles: vec![VertIndex::default(); max_triangles * 3],
            half_edges: vec![None; max_triangles * 3],
            hull: vec![VertIndex::default(); n],
        }
    }

    pub fn edges(&self) -> impl Iterator<Item = Edge<'_>> + '_ {
        self.half_edges
            .iter()
            .enumerate()
            .filter(|(e, opposite)| opposite.map(|o| *o < *e).unwrap_or(true))
            .map(|(e, _)| Edge {
                p0: &self.points[self.triangles[e]],
                p1: &self.points[self.triangles[Self::next_half_edge(e.into())]],
            })
    }

    pub fn next_half_edge(e: EdgeIndex) -> EdgeIndex {
        if *e % 3 == 2 {
            (*e - 2).into()
        } else {
            (*e + 1).into()
        }
    }

    pub fn context(&self) -> HullContext {
        HullContext::new(self.points.len())
    }

    pub fn triangles(&self) -> &[VertIndex] {
        &self.triangles
    }

    pub fn half_edges(&self) -> &[Option<EdgeIndex>] {
        &self.half_edges
    }

    pub fn hull(&self) -> &[VertIndex] {
        &self.hull
    }

    pub fn update(&mut self) {
        let mut hull = self.context();
        self.update_with(&mut hull);
    }

    pub fn update_with(&mut self, hull: &mut HullContext) {
        let mut ids: Vec<VertIndex> = (0..self.points.len()).map(|i| i.into()).collect();
        let hash_size = (self.points.len() as f64).sqrt().ceil();

        let ((p0, p1, p2), seed) = match seed_triangle(&self.points) {
            Ok(v) => v,
            Err(_) => {
                // Degenerate case where all points are in a line
                // Determine if they are linear in x axis or y axis
                let dists = self
                    .points
                    .iter()
                    .map(|p| (p.x - self.points[0].x) + (p.y - self.points[0].y))
                    .collect::<Vec<_>>();

                ids.sort_unstable_by(|&a, &b| dists[a].total_cmp(&dists[b]));

                let mut d0 = f64::NEG_INFINITY;
                for id in ids {
                    let d = dists[id];
                    if d > d0 {
                        self.hull.push(id);
                        d0 = d;
                    }
                }
                return;
            }
        };
        let (i0, i1, i2) = seed.abc();

        let center = circumcenter(p0, p1, p2);

        let dists = self
            .points
            .iter()
            .map(|p| p.distance_squared(center))
            .collect::<Vec<_>>();
        // sort the points by distance from the seed triangle circumcenter
        ids.sort_unstable_by(|&a, &b| dists[a].total_cmp(&dists[b]));

        hull.seed((p0, p1, p2), (i0, i1, i2), center);

        let mut triangles_len = 0;
        self.add_triangle(&mut triangles_len, seed, TriTriple::NONE);

        let mut p_prev = None;
        'a: for i in ids {
            let p = self.points[i];

            // skip near-duplicate points
            if p_prev.map_or(false, |pp| p.distance_squared(pp) <= f64::EPSILON * 2.0) {
                continue;
            }
            p_prev = Some(p);

            // skip seed triangle points
            if i == i0 || i == i1 || i == i2 {
                continue;
            }

            // find a visible edge on the convex hull using edge hash
            let key = hash_key(p, center, hash_size);
            let mut start = Some(0.into());
            for j in 0..(hash_size as usize) {
                start = hull.hash[(key + j) % hash_size as usize];
                if start.map(|s| s != hull.next[s]).unwrap_or(false) {
                    break;
                }
            }

            let sstart = hull.prev[start.unwrap()];
            let mut e = sstart;
            let mut q = hull.next[e];
            while orient2d_fast(p, self.points[e], self.points[q]) >= 0.0 {
                e = q;
                if e == sstart {
                    // likely a near-duplicate point; skip it
                    continue 'a;
                }
                q = hull.next[e];
            }

            // add the first triangle from the point
            let mut t = self.add_triangle(
                &mut triangles_len,
                VertTriple::new(e, i, hull.next[e]),
                TriTriple::new(None, None, Some(hull.tri[e])),
            );

            // recursively flip triangles from the point until they satisfy the Delaunay condition
            hull.tri[i] = self.legalize(hull, t + 2);
            hull.tri[e] = t.into(); // keep track of boundary triangles on the hull
            hull.size += 1;

            // walk forward through the hull, adding more triangles and flipping recursively
            let mut n = hull.next[e];
            q = hull.next[n];
            while orient2d_fast(p, self.points[n], self.points[q]) < 0.0 {
                t = self.add_triangle(
                    &mut triangles_len,
                    VertTriple::new(n, i, q),
                    TriTriple::new(Some(hull.tri[i]), None, Some(hull.tri[n])),
                );
                hull.tri[i] = self.legalize(hull, t + 2);
                hull.next[n] = n;
                hull.size -= 1;
                n = q;
                q = hull.next[n];
            }

            // walk backward from the other side, adding more triangles and flipping
            if e == sstart {
                q = hull.prev[e];
                while orient2d_fast(p, self.points[q], self.points[e]) < 0.0 {
                    t = self.add_triangle(
                        &mut triangles_len,
                        VertTriple::new(q, i, e),
                        TriTriple::new(None, Some(hull.tri[e]), Some(hull.tri[q])),
                    );
                    self.legalize(hull, t + 2);
                    hull.tri[q] = t.into();
                    hull.next[e] = e;
                    hull.size -= 1;
                    e = q;
                    q = hull.next[e];
                }
            }

            // update the hull indices
            hull.start = e;
            hull.prev[i] = e;
            hull.next[e] = i;
            hull.prev[n] = i;
            hull.next[i] = n;

            // save the two new edges in the hash table
            hull.hash[hash_key(p, center, hash_size)] = Some(i);
            hull.hash[hash_key(self.points[e], center, hash_size)] = Some(e);
        }

        let mut e = hull.start;
        for i in 0..hull.size {
            self.hull[i] = e;
            e = hull.next[e];
        }

        self.triangles.truncate(triangles_len);
        self.half_edges.truncate(triangles_len);
        self.hull.truncate(hull.size);
    }

    fn add_triangle(
        &mut self,
        triangles_len: &mut usize,
        vert_ids: VertTriple,
        half_ids: TriTriple,
    ) -> usize {
        let t = *triangles_len;

        self.triangles[t + 0] = vert_ids.a();
        self.triangles[t + 1] = vert_ids.b();
        self.triangles[t + 2] = vert_ids.c();

        self.link(t + 0, half_ids.a());
        self.link(t + 1, half_ids.b());
        self.link(t + 2, half_ids.c());

        *triangles_len += 3;
        t
    }

    fn link(&mut self, a: usize, b: Option<EdgeIndex>) {
        self.half_edges[a] = b;
        if let Some(b) = b {
            self.half_edges[b] = Some(a.into());
        }
    }

    fn legalize(&mut self, hull: &mut HullContext, mut a: usize) -> EdgeIndex {
        let mut i = 0;
        let mut ar;

        // recursion eliminated with a fixed-size stack
        loop {
            let b = self.half_edges[a];

            /* if the pair of triangles doesn't satisfy the Delaunay condition
             * (p1 is inside the circumcircle of [p0, pl, pr]), flip them,
             * then do the same check/flip recursively for the new pair of triangles
             *
             *           pl                    pl
             *          /||\                  /  \
             *       al/ || \bl            al/    \a
             *        /  ||  \              /      \
             *       /  a||b  \    flip    /___ar___\
             *     p0\   ||   /p1   =>   p0\---bl---/p1
             *        \  ||  /              \      /
             *       ar\ || /br             b\    /br
             *          \||/                  \  /
             *           pr                    pr
             */
            let a0 = a - a % 3;
            ar = a0 + (a + 2) % 3;

            let Some(b) = b else {
                // convex hull edge
                if i == 0 {
                    break;
                }
                i -= 1;
                a = hull.edge_stack[i];
                continue;
            };

            let b0 = *b - *b % 3;
            let al = a0 + (a + 1) % 3;
            let bl = (b0 + (*b + 2) % 3).into();

            let p0 = self.triangles[ar];
            let pr = self.triangles[a];
            let pl = self.triangles[al];
            let p1 = self.triangles[bl];

            match in_circle(
                self.points[p0],
                self.points[pr],
                self.points[pl],
                self.points[p1],
            ) {
                true => {
                    self.triangles[a] = p1;
                    self.triangles[b] = p0;

                    let hbl: Option<EdgeIndex> = self.half_edges[bl];

                    // edge swapped on the other side of the hull (rare); fix the halfedge reference
                    if hbl.is_none() {
                        let mut e = hull.start;
                        loop {
                            if hull.tri[e] == bl {
                                hull.tri[e] = a.into();
                                break;
                            }
                            e = hull.prev[e];
                            if e == hull.start {
                                break;
                            }
                        }
                    }
                    self.link(a, hbl);
                    self.link(*b, self.half_edges[ar]);
                    self.link(ar, Some(bl));

                    let br = b0 + (*b + 1) % 3;

                    // don't worry about hitting the cap: it can only happen on extremely degenerate input
                    if i < hull.edge_stack.len() {
                        hull.edge_stack[i] = br;
                        i += 1;
                    }
                }
                false => {
                    if i == 0 {
                        break;
                    }
                    i -= 1;
                    a = hull.edge_stack[i];
                }
            }
        }
        ar.into()
    }
}

#[derive(Debug)]
pub struct HullContext {
    prev: Vec<VertIndex>,
    next: Vec<VertIndex>,
    tri: Vec<EdgeIndex>,
    hash: Vec<Option<VertIndex>>,
    edge_stack: Box<[usize; 256]>,
    hash_size: f64,
    start: VertIndex,
    size: usize,
}

impl HullContext {
    fn new(n: usize) -> Self {
        let hash_size = (n as f64).sqrt().ceil();

        Self {
            prev: vec![VertIndex::default(); n],
            next: vec![VertIndex::default(); n],
            tri: vec![EdgeIndex::default(); n],
            hash: vec![None; hash_size as usize],
            edge_stack: Box::new([0; 256]),
            hash_size,
            start: VertIndex::default(),
            size: 0,
        }
    }

    fn seed(
        &mut self,
        (p0, p1, p2): (DVec2, DVec2, DVec2),
        (i0, i1, i2): (VertIndex, VertIndex, VertIndex),
        center: DVec2,
    ) {
        self.next[i0] = i1;
        self.next[i1] = i2;
        self.next[i2] = i0;

        self.prev[i0] = i2;
        self.prev[i1] = i0;
        self.prev[i2] = i1;

        self.tri[i0] = 0.into();
        self.tri[i1] = 1.into();
        self.tri[i2] = 2.into();

        self.hash.fill(None);
        self.hash[hash_key(p0, center, self.hash_size)] = Some(i0);
        self.hash[hash_key(p1, center, self.hash_size)] = Some(i1);
        self.hash[hash_key(p2, center, self.hash_size)] = Some(i2);

        self.start = i0;
        self.size = 3;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POINTS: [DVec2; 7] = [
        // Outer Square
        DVec2::new(0.0, 0.0),
        DVec2::new(1.0, 0.0),
        DVec2::new(0.0, 1.0),
        DVec2::new(1.0, 1.0),
        // Inner Triangle
        DVec2::new(0.3, 0.4),
        DVec2::new(0.5, 0.7),
        DVec2::new(0.7, 0.4),
    ];

    #[test]
    fn test_circumradius() {
        // Normal case
        assert_eq!(
            circumradius(
                DVec2::new(0.0, 0.0),
                DVec2::new(1.0, 0.0),
                DVec2::new(0.0, 1.0),
            ),
            0.5
        );

        // Degenerate case
        assert!(circumradius(
            DVec2::new(0.0, 0.0),
            DVec2::new(0.0, 1.0),
            DVec2::new(0.0, 2.0),
        )
        .is_nan());
    }

    #[test]
    fn test_circumcenter() {
        // Normal case
        assert_eq!(
            circumcenter(
                DVec2::new(0.0, 0.0),
                DVec2::new(1.0, 0.0),
                DVec2::new(std::f64::consts::FRAC_PI_4, std::f64::consts::FRAC_PI_4),
            ),
            DVec2::new(0.5, 0.2853981633974483)
        );

        // Degenerate case
        assert!(circumcenter(
            DVec2::new(0.0, 0.0),
            DVec2::new(0.0, 1.0),
            DVec2::new(0.0, 2.0),
        )
        .is_nan());
    }

    #[test]
    fn test_seed_triangle() {
        let r = seed_triangle(&POINTS);
        assert_eq!(
            r,
            Ok((
                (POINTS[5], POINTS[4], POINTS[6]),
                VertTriple::new(5.into(), 4.into(), 6.into())
            ))
        )
    }

    #[test]
    fn test_delaunator() {
        let points = POINTS.into();
        let d = Triangulation::new(points);
        assert_eq!(
            d.triangles(),
            [0, 4, 6, 2, 0, 1, 4, 0, 6, 0, 1, 6, 6, 1, 0, 5, 2, 3, 1, 3, 2, 5, 3, 2]
                .into_iter()
                .map(|v| v.into())
                .collect::<Vec<_>>()
        );

        assert_eq!(
            d.half_edges(),
            [
                6isize, 8, 14, -1, 13, 20, 0, 11, 1, -1, 12, 7, 10, 4, 2, 23, 19, 21, -1, 16, 5,
                17, -1, 15
            ]
            .into_iter()
            .map(|v| match v {
                -1 => None,
                _ => Some((v as usize).into()),
            })
            .collect::<Vec<_>>()
        );

        assert_eq!(
            d.hull(),
            [1, 3, 2, 0]
                .into_iter()
                .map(|v| v.into())
                .collect::<Vec<_>>()
        );
    }
}
