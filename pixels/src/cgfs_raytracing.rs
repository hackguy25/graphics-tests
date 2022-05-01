const BACKGROUND_COLOR: (f64, f64, f64) = (0., 0., 0.); // black
const EPS: f64 = 1e-6;

type Sphere = (
    (f64, f64, f64), // center
    f64, // radius
    (f64, f64, f64), // color
    Option<f64>, // specular
    f64 // reflective
);
const SPHERES: [Sphere; 4] = [
    ((0., -1., 3.), 1., (1., 0., 0.), Some(500.), 0.2),
    ((2., 0., 4.), 1., (0., 0., 1.), Some(500.), 0.3),
    ((-2., 0., 4.), 1., (0., 1., 0.), Some(10.), 0.4),
    ((0., -5001., 0.), 5000., (1., 1., 0.), Some(1000.), 0.5),
];

enum Light {
    Ambient(f64),
    Point(f64, (f64, f64, f64)),
    Directional(f64, (f64, f64, f64)),
}
const LIGHTS: [Light; 3] = [
    Light::Ambient(0.2),
    Light::Point(0.6, (2., 1., 0.)),
    Light::Directional(0.2, (1., 4., 4.)),
];

/// Projects a 3D point onto the viewport.
pub fn canvas_to_viewport(v: &(f64, f64, f64), x: f64, y: f64, w: f64, h: f64) -> (f64, f64, f64) {
    (x * v.0 / w, y * v.1 / h, v.2)
}

/// Helper for dot product calculation.
macro_rules! dot3 {
    ($a: ident, $b: ident) => {
        $a.0 * $b.0 + $a.1 * $b.1 + $a.2 * $b.2
    };
}

/// Helper for vector length calculation.
macro_rules! len3 {
    ($a: ident) => {
        dot3!($a, $a).sqrt()
    };
}

/// Helper for vector normalization.
macro_rules! norm3 {
    ($a: ident) => {{
        let len = dot3!($a, $a).sqrt();
        ($a.0 / len, $a.1 / len, $a.2 / len)
    }};
}

/// Helper for linear interpolation of vectors.
macro_rules! larp3 {
    ($a: ident, $b: ident, $r: expr) => {{
        let r = $r;
        let rc = 1. - r;
        (
            $a.0 * rc + $b.0 * r,
            $a.1 * rc + $b.1 * r,
            $a.2 * rc + $b.2 * r
        )
    }};
}

/// Calculate the two intersections of a ray with a sphere.
fn intersect_ray_sphere(o: &(f64, f64, f64), d: &(f64, f64, f64), sphere: &Sphere) -> (f64, f64) {
    let r = sphere.1;
    let c0 = (o.0 - sphere.0 .0, o.1 - sphere.0 .1, o.2 - sphere.0 .2);

    let a = dot3!(d, d);
    let b = 2. * dot3!(c0, d);
    let c = dot3!(c0, c0) - r * r;

    let discriminant = b * b - 4. * a * c;
    if discriminant < 0. {
        (f64::INFINITY, f64::INFINITY)
    } else {
        (
            (-b + discriminant.sqrt()) / (2. * a),
            (-b - discriminant.sqrt()) / (2. * a),
        )
    }
}

/// Reflect a ray off a surface.
fn reflect_ray(r: &(f64, f64, f64), n: &(f64, f64, f64)) -> (f64, f64, f64) {
    let n_dot_r = dot3!(n, r);
    (
        2. * n.0 * n_dot_r - r.0,
        2. * n.1 * n_dot_r - r.1,
        2. * n.2 * n_dot_r - r.2,
    )
}

/// Computes a lighting at a given point in space.
fn compute_lighting(
    p: &(f64, f64, f64),
    n: &(f64, f64, f64),
    v: &(f64, f64, f64),
    s: Option<f64>,
) -> f64 {
    let mut i = 0.;
    for light in LIGHTS.iter() {
        if let Light::Ambient(intensity) = light {
            i += intensity;
        } else {
            let (intensity, l, t_max) = match light {
                Light::Point(intensity, position) => (
                    intensity,
                    (position.0 - p.0, position.1 - p.1, position.2 - p.2),
                    1.,
                ),
                Light::Directional(intensity, direction) => (intensity, *direction, f64::INFINITY),
                _ => panic!("Never"),
            };

            // Shadow check
            let (shadow_sphere, _) = closest_intersection(p, &l, EPS, t_max);
            if let None = shadow_sphere {
                // Diffuse
                let n_dot_l = dot3!(n, l);
                if n_dot_l > 0. {
                    i += intensity * n_dot_l / (len3!(n) * len3!(l));
                }

                // Specular
                if let Some(s) = s {
                    let r = reflect_ray(&l, n);
                    let r_dot_v = dot3!(r, v);
                    if r_dot_v > 0. {
                        let powf = r_dot_v / (len3!(r) * len3!(v));
                        i += intensity * powf.powf(s);
                    }
                }
            }
        }
    }
    i
}

/// Determine which sphere intersects the ray first.
fn closest_intersection<'a>(
    o: &(f64, f64, f64),
    d: &(f64, f64, f64),
    t_min: f64,
    t_max: f64,
) -> (Option<&'a Sphere>, f64) {
    let mut closest_t = f64::INFINITY;
    let mut closest_sphere: Option<&Sphere> = None;
    for sphere in SPHERES.iter() {
        let (t1, t2) = intersect_ray_sphere(o, d, sphere);
        if t_min <= t1 && t1 <= t_max && t1 < closest_t {
            closest_t = t1;
            closest_sphere = Some(sphere)
        }
        if t_min <= t2 && t2 <= t_max && t2 < closest_t {
            closest_t = t2;
            closest_sphere = Some(sphere)
        }
    }
    (closest_sphere, closest_t)
}

/// Trace a ray.
pub fn trace_ray(
    o: &(f64, f64, f64),
    d: &(f64, f64, f64),
    t_min: f64,
    t_max: f64,
    recursion_depth: u64,
) -> (f64, f64, f64) {
    let (closest_sphere, closest_t) = closest_intersection(o, d, t_min, t_max);
    if let Some(s) = closest_sphere {
        // Compute local color
        let p = (
            o.0 + closest_t * d.0,
            o.1 + closest_t * d.1,
            o.2 + closest_t * d.2,
        );
        let n = (p.0 - s.0 .0, p.1 - s.0 .1, p.2 - s.0 .2);
        let n = norm3!(n);
        let i = compute_lighting(&p, &n, &(-d.0, -d.1, -d.2), s.3);
        let local_color = (s.2 .0 * i, s.2 .1 * i, s.2 .2 * i);
        if recursion_depth == 0 || s.4 <= 0. {
            local_color
        } else {
            // Compute the reflected color
            let r = (-d.0, -d.1, -d.2);
            let r = reflect_ray(&r, &n);
            let reflected_color = trace_ray(&p, &r, EPS, f64::INFINITY, recursion_depth - 1);
            larp3!(local_color, reflected_color, s.4)
        }
    } else {
        BACKGROUND_COLOR
    }
}
