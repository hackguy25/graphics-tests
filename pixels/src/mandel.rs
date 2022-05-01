use nalgebra::Complex;

/// Map the iteration number to a shade of grey.
pub fn gradient_bw(iters: f64, max_iters: f64) -> [u8; 4] {
    let gradient = ((iters / max_iters).clamp(0., 0.9999) * 256.) as u8;
    [gradient, gradient, gradient, 0xff]
}

/// Map the iteration number to a shade between black, blue, magenta and white.
pub fn gradient_bbmw(iters: f64, max_iters: f64) -> [u8; 4] {
    let gradient = (iters / max_iters).clamp(0., 0.9999) * 3.;
    if gradient < 1. {
        let gradient = (gradient * 256.) as u8;
        [0x00, 0x00, gradient, 0xff]
    } else if gradient < 2. {
        let gradient = ((gradient - 1.) * 256.) as u8;
        [gradient, 0x00, 0xff, 0xff]
    } else {
        let gradient = ((gradient - 2.) * 256.) as u8;
        [0xff, gradient, 0xff, 0xff]
    }
}

/// Apply the iteration function until the norm is larger than 4 or maximum number of iterations is hit.
/// Return the number of iterations required.
pub fn mandelbrot_naive(c: Complex<f64>, max_iters: u64) -> u64 {
    let mut z = Complex::new(0., 0.);
    let mut i = 0;
    loop {
        if z.norm_sqr() > 4. || i >= max_iters {
            break
        }
        z = z * z + c;
        i += 1;
    }
    i
}

/// Apply the iteration function until the norm is larger than 4 or maximum number of iterations is hit.
/// Return a smoothed value that represents wheter the last iteration or the one before was closer to 4.
pub fn mandelbrot_smooth(c: Complex<f64>, max_iters: u64) -> f64 {
    let mut z = Complex::new(0., 0.);
    let mut z_p = Complex::new(0., 0.);
    let mut i = 0;
    loop {
        if z.norm_sqr() > 4. || i >= max_iters {
            break
        }
        z_p = z;
        z = z * z + c;
        i += 1;
    }
    let pow = -1.;
    let f_z = (z.norm_sqr() - 3.9999999).abs().powf(pow);
    let f_z_p = (z_p.norm_sqr() - 4.0000001).abs().powf(pow);
    ((i as f64 - 1.) * f_z_p + i as f64 * f_z) / (f_z_p + f_z)
}

/// Iterator that generates the 2D Halton sequence.
pub struct Halton2D<const B1: usize, const B2: usize> {
    n1: usize,
    n2: usize,
    d1: usize,
    d2: usize,
}

impl<const B1: usize, const B2: usize> Iterator for Halton2D<B1, B2> {
    type Item = (f64, f64);
    /// Adapted from https://en.wikipedia.org/wiki/Halton_sequence#Implementation
    fn next(&mut self) -> Option<Self::Item> {
        let x1 = self.d1 - self.n1;
        if x1 == 1 {
            self.n1 = 1;
            self.d1 *= B1;
        } else {
            let mut y1 = self.d1 / B1;
            while x1 <= y1 {
                y1 /= B1
            }
            self.n1 = (B1 + 1) * y1 - x1
        }
        let x2 = self.d2 - self.n2;
        if x2 == 1 {
            self.n2 = 1;
            self.d2 *= B2;
        } else {
            let mut y2 = self.d2 / B2;
            while x2 <= y2 {
                y2 /= B2
            }
            self.n2 = (B2 + 1) * y2 - x2
        }
        Some((self.n1 as f64 / self.d1 as f64, self.n2 as f64 / self.d2 as f64))
    }
}

pub fn halton_2d<const B1: usize, const B2: usize>() -> Halton2D<B1, B2> {
    Halton2D {
        d1: 1,
        d2: 1,
        n1: 0,
        n2: 0
    }
}