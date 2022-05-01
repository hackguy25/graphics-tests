pub const TRIANGLE_POINTS: &[(i64, i64)] = &[(-200, -100), (240, 120), (-50, -200)];
pub const TRIANGLE: &[(usize, usize)] = &[(0, 1), (1, 2), (2, 0)];
pub const CUBE_POINTS: &[(f64, f64, f64)] = &[
    (-2., -0.5, 5.), // vAf
    (-2., 0.5, 5.),  // vBf
    (-1., 0.5, 5.),  // vCf
    (-1., -0.5, 5.), // vDf
    (-2., -0.5, 6.), // vAb
    (-2., 0.5, 6.),  // vBb
    (-1., 0.5, 6.),  // vCb
    (-1., -0.5, 6.), // vDb
];
pub const CUBE: &[((usize, usize), (f64, f64, f64))] = &[
    ((0, 1), (0., 0., 1.)),
    ((1, 2), (0., 0., 1.)),
    ((2, 3), (0., 0., 1.)),
    ((3, 0), (0., 0., 1.)),
    ((4, 5), (1., 0., 0.)),
    ((5, 6), (1., 0., 0.)),
    ((6, 7), (1., 0., 0.)),
    ((7, 4), (1., 0., 0.)),
    ((0, 4), (0., 1., 0.)),
    ((1, 5), (0., 1., 0.)),
    ((2, 6), (0., 1., 0.)),
    ((3, 7), (0., 1., 0.)),
];

/// Helper for swapping two values if the condition is true.
macro_rules! swap_if {
    ($a: ident, $b: ident, $c: expr) => {
        if $c {
            ($b, $a)
        } else {
            ($a, $b)
        }
    };
}

/// Helper for an inline `if` statement, for cosmetic reasons.
macro_rules! inline_if {
    ($a: expr, $b: expr, $c: expr) => {
        if $c {
            $a
        } else {
            $b
        }
    };
}

/// Iterator over an interpolated range.
/// Can provide rounded or truncated values.
struct Interpolate<Return, const ROUNDED: bool> {
    i: i64,
    i1: i64,
    d: f64,
    a: f64,
    output: std::marker::PhantomData<Return>,
}

impl<const ROUNDED: bool> Iterator for Interpolate<f64, ROUNDED> {
    type Item = (i64, f64);
    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.i1 {
            self.i += 1;
            self.d += self.a;
            if ROUNDED {
                Some((self.i, self.d.round()))
            } else {
                Some((self.i, self.d))
            }
        } else {
            None
        }
    }
}

impl<const ROUNDED: bool> Iterator for Interpolate<i64, ROUNDED> {
    type Item = (i64, i64);
    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.i1 {
            self.i += 1;
            self.d += self.a;
            if ROUNDED {
                Some((self.i, self.d.round() as i64))
            } else {
                Some((self.i, self.d as i64))
            }
        } else {
            None
        }
    }
}

impl<Return, const ROUNDED: bool> Interpolate<Return, ROUNDED> {
    fn left_of(&self, other: &Self) -> bool {
        self.a < other.a
    }
}

/// Linear interpolation between (i0, d0) and (i1, d1) with integer steps for i.
fn interpolate(i0: i64, d0: f64, i1: i64, d1: f64) -> Interpolate<f64, false> {
    let a = (d1 - d0) as f64 / (i1 - i0) as f64;
    Interpolate::<f64, false> {
        i: i0 - 1,
        i1,
        d: d0 as f64 - a,
        a,
        output: std::marker::PhantomData,
    }
}

/// Linear interpolation between (i0, d0) and (i1, d1) with integer steps for i and rounded values for d.
fn interpolate_rounded(i0: i64, d0: i64, i1: i64, d1: i64) -> Interpolate<i64, true> {
    let a = (d1 - d0) as f64 / (i1 - i0) as f64;
    Interpolate::<i64, true> {
        i: i0 - 1,
        i1,
        d: d0 as f64 - a,
        a,
        output: std::marker::PhantomData,
    }
}

/// Linear interpolation between (i0, d0, z0) and (i1, d1, z1) with integer steps for i and rounded values for d.
fn interpolate_rounded_with_depth(
    i0: i64,
    d0: i64,
    z0: f64,
    i1: i64,
    d1: i64,
    z1: f64,
) -> (Interpolate<i64, true>, Interpolate<f64, false>) {
    let d = {
        let a = (d1 - d0) as f64 / (i1 - i0) as f64;
        Interpolate::<i64, true> {
            i: i0 - 1,
            i1,
            d: d0 as f64 - a,
            a,
            output: std::marker::PhantomData,
        }
    };
    let z = {
        let a = (z1 - z0) / (i1 - i0) as f64;
        Interpolate::<f64, false> {
            i: i0 - 1,
            i1,
            d: z0,
            a,
            output: std::marker::PhantomData,
        }
    };
    (d, z)
}

/// Given two points on the canvas, draw a line between them.
pub fn draw_line<PutPixel>(
    p0: &(i64, i64),
    p1: &(i64, i64),
    color: &(f64, f64, f64),
    mut put_pixel: PutPixel,
) where
    PutPixel: FnMut(i64, i64, &(f64, f64, f64)),
{
    if (p1.0 - p0.0).abs() > (p1.1 - p0.1).abs() {
        // Line is horizontal-ish
        // Make sure x0 < x1
        let (p0, p1) = swap_if!(p0, p1, p0.0 > p1.0);
        for (x, y) in interpolate_rounded(p0.0, p0.1, p1.0, p1.1) {
            put_pixel(x, y, color);
        }
    } else {
        // Line is vertical-ish
        // Make sure y0 < y1
        let (p0, p1) = if p0.1 > p1.1 { (p1, p0) } else { (p0, p1) };
        for (y, x) in interpolate_rounded(p0.1, p0.0, p1.1, p1.0) {
            put_pixel(x, y, color);
        }
    }
}

/// Given three points on the canvas, draw a triangle defined by them.
pub fn draw_filled_triangle<PutPixel>(
    p0: &(i64, i64),
    p1: &(i64, i64),
    p2: &(i64, i64),
    color: &(f64, f64, f64),
    mut put_pixel: PutPixel,
) where
    PutPixel: FnMut(i64, i64, &(f64, f64, f64)),
{
    // Sort the points so that y0 <= y1 <= y2
    let (p0, p1) = swap_if!(p0, p1, p1.1 < p0.1);
    let (p0, p2) = swap_if!(p0, p2, p2.1 < p0.1);
    let (p1, p2) = swap_if!(p1, p2, p2.1 < p1.1);

    // Compute the x coordinates of the triangle edges
    let x01 = interpolate_rounded(p0.1, p0.0, p1.1, p1.0);
    let mut x12 = interpolate_rounded(p1.1, p1.0, p2.1, p2.0);
    let x02 = interpolate_rounded(p0.1, p0.0, p2.1, p2.0);

    // Concatenate the short sides
    let left = x02.left_of(&x01); // hack since we can't randomly access iterator values
    x12.next();
    let x012 = x01.chain(x12);
    let shim = interpolate_rounded(0, 0, 0, 0); // placeholder so x012 and x02 are of the same type
    let x02 = x02.chain(shim);

    // Determine which is left and which is right
    let lines = if left { x02.zip(x012) } else { x012.zip(x02) };

    // Draw the horizontal segments
    for ((y, x_l), (_, x_r)) in lines {
        for x in x_l..=x_r {
            put_pixel(x, y, color)
        }
    }
}

/// Given three points on the canvas, draw a triangle defined by them where it is not obstructed.
pub fn draw_filled_triangle_with_depth<PutPixel>(
    p0: &(i64, i64, f64),
    p1: &(i64, i64, f64),
    p2: &(i64, i64, f64),
    color: &(f64, f64, f64),
    mut put_pixel: PutPixel,
) where
    PutPixel: FnMut(i64, i64, f64, &(f64, f64, f64)),
{
    // Sort the points so that y0 <= y1 <= y2
    let (p0, p1) = swap_if!(p0, p1, p1.1 < p0.1);
    let (p0, p2) = swap_if!(p0, p2, p2.1 < p0.1);
    let (p1, p2) = swap_if!(p1, p2, p2.1 < p1.1);

    // Compute the x coordinates of the triangle edges
    let x01 = interpolate_rounded_with_depth(p0.1, p0.0, 1. / p0.2, p1.1, p1.0, 1. / p1.2);
    let mut x12 = interpolate_rounded_with_depth(p1.1, p1.0, 1. / p1.2, p2.1, p2.0, 1. / p2.2);
    let x02 = interpolate_rounded_with_depth(p0.1, p0.0, 1. / p0.2, p2.1, p2.0, 1. / p2.2);

    // Concatenate the short sides
    let left = x02.0.left_of(&x01.0); // hack since we can't randomly access iterator values
    (x12.0.next(), x12.1.next());
    let x012 = (x01.0.chain(x12.0), x01.1.chain(x12.1));
    let shim = interpolate_rounded_with_depth(0, 0, 0., 0, 0, 0.); // placeholder so x012 and x02 are of the same type
    let x02 = (x02.0.chain(shim.0), x02.1.chain(shim.1));

    // Determine which is left and which is right
    let lines = {
        let x02 = x02.0.zip(x02.1);
        let x012 = x012.0.zip(x012.1);
        if left { x02.zip(x012) } else { x012.zip(x02) }
    };

    // Draw the horizontal segments
    for (((y, x_l), (_, z_l)), ((_, x_r), (_, z_r))) in lines {
        for (x, z) in interpolate(x_l, z_l, x_r, z_r) {
            put_pixel(x, y, z, color)
        }
    }
}

/// Given three points on the canvas, draw a shaded triangle defined by them.
pub fn draw_shaded_triangle<PutPixel>(
    p0: &(i64, i64),
    p1: &(i64, i64),
    p2: &(i64, i64),
    color: &(f64, f64, f64),
    h: &(f64, f64, f64),
    mut put_pixel: PutPixel,
) where
    PutPixel: FnMut(i64, i64, &(f64, f64, f64)),
{
    // Sort the points so that y0 <= y1 <= y2
    let h = inline_if!((h.1, h.0, h.2), (h.0, h.1, h.2), p1.1 < p0.1);
    let (p0, p1) = swap_if!(p0, p1, p1.1 < p0.1);
    let h = inline_if!((h.2, h.1, h.0), (h.0, h.1, h.2), p2.1 < p0.1);
    let (p0, p2) = swap_if!(p0, p2, p2.1 < p0.1);
    let h = inline_if!((h.0, h.2, h.1), (h.0, h.1, h.2), p2.1 < p1.1);
    let (p1, p2) = swap_if!(p1, p2, p2.1 < p1.1);

    // Compute the x coordinates and h values of the triangle edges
    let x01 = interpolate_rounded(p0.1, p0.0, p1.1, p1.0);
    let h01 = interpolate(p0.1, h.0, p1.1, h.1);
    let mut x12 = interpolate_rounded(p1.1, p1.0, p2.1, p2.0);
    let mut h12 = interpolate(p1.1, h.1, p2.1, h.2);
    let x02 = interpolate_rounded(p0.1, p0.0, p2.1, p2.0);
    let h02 = interpolate(p0.1, h.0, p2.1, h.2);

    // Concatenate the short sides
    let left = x02.left_of(&x01); // hack since we can't randomly access iterator values
    x12.next();
    let x012 = x01.chain(x12);
    let shim = interpolate_rounded(0, 0, 0, 0); // placeholder so x012 and x02 are of the same type
    let x02 = x02.chain(shim);

    h12.next();
    let h012 = h01.chain(h12);
    let shim = interpolate(0, 0., 0, 0.); // placeholder so h012 and h02 are of the same type
    let h02 = h02.chain(shim);

    // Determine which is left and which is right
    let lines = if left {
        x02.zip(h02).zip(x012.zip(h012))
    } else {
        x012.zip(h012).zip(x02.zip(h02))
    };

    // Draw the horizontal segments
    for (((y, x_l), (_, h_l)), ((_, x_r), (_, h_r))) in lines {
        for (x, h) in interpolate(x_l, h_l, x_r, h_r) {
            let shaded_color = (color.0 * h, color.1 * h, color.2 * h);
            put_pixel(x, y, &shaded_color)
        }
    }
}

/// Project a point on the viewport to a point on the canvas.
pub fn viewport_to_canvas(v: &(f64, f64, f64), w: f64, h: f64, x: f64, y: f64) -> (i64, i64) {
    ((x * w / v.0) as i64, (y * h / v.1) as i64)
}

/// Project a point in space to a point on the canvas.
pub fn project_vertex(
    v: &(f64, f64, f64),
    o: &(f64, f64, f64),
    w: f64,
    h: f64,
    p: &(f64, f64, f64),
) -> (i64, i64) {
    viewport_to_canvas(
        v,
        w,
        h,
        (p.0 - o.0) * v.2 / (p.2 - o.2),
        (p.1 - o.1) * v.2 / (p.2 - o.2),
    )
}
