use glam::DVec2;

use super::VertTriple;

pub(crate) fn seed_triangle(points: &[DVec2]) -> Result<((DVec2, DVec2, DVec2), VertTriple), ()> {
    // Calulate bounding box
    let (bb_min, bb_max) = points.iter().fold(
        (DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)),
        |(min, max), v| (min.min(*v), max.max(*v)),
    );
    let c = (bb_min + bb_max) / 2.0;

    // pick a seed point closest to the center
    let (i0, p0, _) = points.iter().enumerate().fold(
        (0, DVec2::NAN, f64::INFINITY),
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
        (0, DVec2::NAN, f64::INFINITY),
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
            (0, DVec2::NAN, f64::INFINITY),
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
    if orient2d_fast(p0, p1, p2) < 0.0 {
        std::mem::swap(&mut i1, &mut i2);
        std::mem::swap(&mut p1, &mut p2);
    }

    Ok((
        (p0, p1, p2),
        VertTriple::new(i0.into(), i1.into(), i2.into()),
    ))
}

pub(crate) fn circumradius(a: DVec2, b: DVec2, c: DVec2) -> f64 {
    let d = b - a;
    let e = c - a;

    let bl = d.length_squared();
    let cl = e.length_squared();
    let dia = 0.5 / (d.x * e.y - d.y * e.x);

    let x = (e.y * bl - d.y * cl) * dia;
    let y = (d.x * cl - e.x * bl) * dia;

    x * x + y * y
}

pub(crate) fn circumcenter(a: DVec2, b: DVec2, c: DVec2) -> DVec2 {
    let d = b - a;
    let e = c - a;

    let bl = d.length_squared();
    let cl = e.length_squared();
    let dia = 0.5 / (d.x * e.y - d.y * e.x);

    let x = a.x + (e.y * bl - d.y * cl) * dia;
    let y = a.y + (d.x * cl - e.x * bl) * dia;

    DVec2::new(x, y)
}

pub(crate) fn in_circle(a: DVec2, b: DVec2, c: DVec2, p: DVec2) -> bool {
    let d = a - p;
    let e = b - p;
    let f = c - p;

    let ap = d.length_squared();
    let bp = e.length_squared();
    let cp = f.length_squared();

    (d.x * (e.y * cp - bp * f.y) - d.y * (e.x * cp - bp * f.x) + ap * (e.x * f.y - e.y * f.x)) < 0.0
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

pub(crate) fn hash_key(p: DVec2, c: DVec2, hash_size: f64) -> usize {
    ((pseudo_angle(p.x - c.x, p.y - c.y) * hash_size).floor() % hash_size) as usize
}

pub fn orient2d_fast(a: DVec2, b: DVec2, c: DVec2) -> f64 {
    let acx = a[0] - c[0];
    let bcx = b[0] - c[0];
    let acy = a[1] - c[1];
    let bcy = b[1] - c[1];
    acx * bcy - acy * bcx
}
