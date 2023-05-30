use crate::types::{VertTriple, Vertex};

pub(crate) fn seed_triangle(
    points: &[Vertex],
) -> Result<((Vertex, Vertex, Vertex), VertTriple), ()> {
    // Calulate bounding box
    let (bb_min, bb_max) = points.iter().fold(
        (
            Vertex::splat(f64::INFINITY),
            Vertex::splat(f64::NEG_INFINITY),
        ),
        |(min, max), v| (min.min(*v), max.max(*v)),
    );
    let c = (bb_min + bb_max) / 2.0;

    // pick a seed point closest to the center
    let (i0, p0, _) = points.iter().enumerate().fold(
        (0, Vertex::NAN, f64::INFINITY),
        |(i_min, p_min, d_min), (i, p)| {
            let d = p.distance_squared(c);
            if d < d_min {
                (i, *p, d)
            } else {
                (i_min, p_min, d_min)
            }
        },
    );

    // Find the closest point to the seed
    let (mut i1, mut p1, _) = points.iter().enumerate().filter(|(i, _)| *i != i0).fold(
        (0, Vertex::NAN, f64::INFINITY),
        |(i_min, p_min, d_min), (i, p)| {
            let d = p.distance_squared(p0);
            if d < d_min {
                (i, *p, d)
            } else {
                (i_min, p_min, d_min)
            }
        },
    );

    // Find the 3rd poing of seed triangle
    let (mut i2, mut p2, r_min) = points
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != i0 && *i != i1)
        .fold(
            (0, Vertex::NAN, f64::INFINITY),
            |(i_min, p_min, r_min), (i, p)| {
                let r = circumradius(p0, p1, *p);
                if r < r_min {
                    (i, *p, r)
                } else {
                    (i_min, p_min, r_min)
                }
            },
        );

    if r_min.is_subnormal() {
        return Err(());
    }

    // swap the order of the seed points for counter-clockwise orientation
    if orient2d_fast(p0.into(), p1.into(), p2.into()) < 0.0 {
        std::mem::swap(&mut i1, &mut i2);
        std::mem::swap(&mut p1, &mut p2);
    }

    Ok((
        (p0, p1, p2),
        VertTriple::new(i0.into(), i1.into(), i2.into()),
    ))
}

pub(crate) fn circumradius(a: Vertex, b: Vertex, c: Vertex) -> f64 {
    let d = b - a;
    let e = c - a;

    let bl = d.length_squared();
    let cl = e.length_squared();
    let dia = 0.5 / (d.x() * e.y() - d.y() * e.x());

    let x = (e.y() * bl - d.y() * cl) * dia;
    let y = (d.x() * cl - e.x() * bl) * dia;

    x * x + y * y
}

pub(crate) fn circumcenter(a: Vertex, b: Vertex, c: Vertex) -> Vertex {
    let d = b - a;
    let e = c - a;

    let bl = d.length_squared();
    let cl = e.length_squared();
    let dia = 0.5 / (d.x() * e.y() - d.y() * e.x());

    let x = a.x() + (e.y() * bl - d.y() * cl) * dia;
    let y = a.y() + (d.x() * cl - e.x() * bl) * dia;

    Vertex::new(x, y)
}

pub(crate) fn in_circle(a: Vertex, b: Vertex, c: Vertex, p: Vertex) -> bool {
    let d = a - p;
    let e = b - p;
    let f = c - p;

    let ap = d.length_squared();
    let bp = e.length_squared();
    let cp = f.length_squared();

    (d.x() * (e.y() * cp - bp * f.y()) - d.y() * (e.x() * cp - bp * f.x())
        + ap * (e.x() * f.y() - e.y() * f.x()))
        < 0.0
}

// monotonically increases with real angle, but doesn't need expensive trigonometry
pub(crate) fn pseudo_angle(dx: f64, dy: f64) -> f64 {
    let p = dx / (dx.abs() + dy.abs());

    // [0..1]
    if dy > 0.0 {
        (3.0 - p) / 4.0
    } else {
        (1.0 + p) / 4.0
    }
}

pub(crate) fn hash_key(p: Vertex, c: Vertex, hash_size: f64) -> usize {
    ((pseudo_angle(p.x() - c.x(), p.y() - c.y()) * hash_size).floor() % hash_size) as usize
}

pub fn orient2d_fast(a: Vertex, b: Vertex, c: Vertex) -> f64 {
    (a.y() - c.y()) * (b.x() - c.x()) - (a.x() - c.x()) * (b.y() - c.y())
}
