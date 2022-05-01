#![allow(dead_code)]

use nalgebra::{Matrix3, Matrix3x4, Matrix4, Point3, Vector3, Vector4};

pub struct Model<'a> {
    pub vertices: &'a [Vector4<f64>],
    pub triangles: &'a [(usize, usize, usize)],
    pub triangle_colors: &'a [(f64, f64, f64)],
}

const CUBE: Model = Model {
    vertices: &[
        Vector4::new(1., 1., 1., 1.),
        Vector4::new(-1., 1., 1., 1.),
        Vector4::new(-1., -1., 1., 1.),
        Vector4::new(1., -1., 1., 1.),
        Vector4::new(1., 1., -1., 1.),
        Vector4::new(-1., 1., -1., 1.),
        Vector4::new(-1., -1., -1., 1.),
        Vector4::new(1., -1., -1., 1.),
    ],
    triangles: &[
        (0, 1, 2),
        (0, 2, 3),
        (4, 0, 3),
        (4, 3, 7),
        (5, 4, 7),
        (5, 7, 6),
        (1, 5, 6),
        (1, 6, 2),
        (4, 5, 1),
        (4, 1, 0),
        (2, 6, 7),
        (2, 7, 3),
    ],
    triangle_colors: &[
        (1., 0., 0.),
        (1., 0., 0.),
        (0., 1., 0.),
        (0., 1., 0.),
        (0., 0., 1.),
        (0., 0., 1.),
        (1., 1., 0.),
        (1., 1., 0.),
        (1., 0., 1.),
        (1., 0., 1.),
        (0., 1., 1.),
        (0., 1., 1.),
    ],
};

pub fn homogeneous_rotation(x: f64, y: f64, z: f64) -> Matrix4<f64> {
    Matrix4::new_rotation(Vector3::new(x, y, z))
}

pub fn homogeneous_scale(x: f64, y: f64, z: f64) -> Matrix4<f64> {
    Matrix4::from_diagonal(&Vector4::new(x, y, z, 1.))
}

pub fn homogeneous_scale_vector(scale: Vector3<f64>) -> Matrix4<f64> {
    Matrix4::from_diagonal(&Vector4::new(scale[0], scale[1], scale[2], 1.))
}

pub fn homogeneous_translation(x: f64, y: f64, z: f64) -> Matrix4<f64> {
    let mut ret = Matrix4::identity();
    ret.fixed_slice_mut::<3, 1>(0, 3)
        .set_column(0, &Vector3::new(x, y, z));
    ret
}

pub fn homogeneous_translation_vector(translation: Vector3<f64>) -> Matrix4<f64> {
    let mut ret = Matrix4::identity();
    ret.fixed_slice_mut::<3, 1>(0, 3)
        .set_column(0, &translation);
    ret
}

pub fn homogeneous_projection(d: f64) -> Matrix3x4<f64> {
    Matrix3x4::new(d, 0., 0., 0., 0., d, 0., 0., 0., 0., 1., 0.)
}

pub fn homogeneous_viewport_to_homogeneous_canvas(s_x: f64, s_y: f64) -> Matrix3<f64> {
    Matrix3::from_diagonal(&Vector3::new(s_x, s_y, 1.))
}

/// Project a homogenous point in space to a homogenous point on the canvas.
pub fn homogeneous_3d_to_canvas(
    v_w: f64,
    v_h: f64,
    d: f64,
    c_w: f64,
    c_h: f64,
    point: Vector4<f64>,
) -> (i64, i64, f64) {
    (
        ((point[0] * d * c_w) / (point[2] * v_w)) as i64,
        ((point[1] * d * c_h) / (point[2] * v_h)) as i64,
        point[2],
    )
}

/// A instance of a model.
pub struct Instance<'a> {
    pub model: &'a Model<'a>,
    scale: Vector3<f64>,
    rotation: Matrix4<f64>,
    position: Vector3<f64>,
    transform: Matrix4<f64>,
}

impl<'a> Instance<'a> {
    fn new(
        model: &'a Model<'a>,
        scale: Vector3<f64>,
        rotation: Matrix4<f64>,
        position: Vector3<f64>,
    ) -> Instance<'a> {
        Instance {
            model,
            scale,
            rotation,
            position,
            transform: homogeneous_translation_vector(position)
                * rotation
                * homogeneous_scale_vector(scale),
        }
    }

        pub fn update_scale(&mut self, scale: Vector3<f64>) {
        self.scale = scale;
        self.transform = homogeneous_translation_vector(self.position)
            * self.rotation
            * homogeneous_scale_vector(self.scale)
    }

        pub fn update_rotation(&mut self, rotation: Matrix4<f64>) {
        self.rotation = rotation;
        self.transform = homogeneous_translation_vector(self.position)
            * self.rotation
            * homogeneous_scale_vector(self.scale)
    }

        pub fn update_position(&mut self, position: Vector3<f64>) {
        self.position = position;
        self.transform = homogeneous_translation_vector(self.position)
            * self.rotation
            * homogeneous_scale_vector(self.scale)
    }

    pub fn transform(&self) -> Matrix4<f64> {
        self.transform
    }

    pub fn radius_from_origin(&self) -> f64 {
        self.model.vertices.iter().fold(0., |maximum, vertex| {
            (0..3)
                .map(|i| vertex[i] * self.scale[i] / vertex[3])
                .map(|x| x * x)
                .sum::<f64>()
                .sqrt()
                .max(maximum)
        })
    }

    pub fn project(
        &'a self,
        camera_m_inv: &Matrix4<f64>,
        v_w: f64,
        v_h: f64,
        d: f64,
        c_w: f64,
        c_h: f64,
    ) -> Triangles<'a> {
        macro_rules! homogenize {
            ($a: expr) => {
                homogeneous_3d_to_canvas(v_w, v_h, d, c_w, c_h, $a)
            };
        }
        let m = camera_m_inv * self.transform();
        let projected = self
            .model
            .vertices
            .iter()
            .map(|x| homogenize!(m * x))
            .collect::<Vec<_>>();
        Triangles::Full {
            instance: self,
            projected,
            next_idx: 0,
        }
    }

    pub fn project_and_clip(
        &'a self,
        camera_m_inv: &Matrix4<f64>,
        v_w: f64,
        v_h: f64,
        d: f64,
        c_w: f64,
        c_h: f64,
    ) -> Triangles<'a> {
        let radius = self.radius_from_origin();
        let clipping_planes = vec![
            (Vector3::new(0., 0., 1.), d),
            (Vector3::new(d, 0., v_w / 2.).normalize(), 0.),
            (Vector3::new(-d, 0., v_w / 2.).normalize(), 0.),
            (Vector3::new(0., d, v_h / 2.).normalize(), 0.),
            (Vector3::new(0., -d, v_h / 2.).normalize(), 0.),
        ];
        let center =
            Vector3::from_homogeneous(camera_m_inv * self.position.to_homogeneous()).unwrap();
        // println!("center: {center}");
        let worst_position = clipping_planes
            .iter()
            .map(|(n, d)| {
                let ret = center.dot(n) + d;
                // println!("distance: {ret}");
                ret
            })
            .fold(f64::INFINITY, |a, b| a.min(b));
        // println!("worst: {worst_position}");
        // println!("radius: {radius}");
        let m = camera_m_inv * self.transform();
        macro_rules! homogenize {
            ($a: expr) => {
                homogeneous_3d_to_canvas(v_w, v_h, d, c_w, c_h, $a)
            };
        }
        if worst_position <= -radius {
            // println!("Empty");
            Triangles::Empty
        } else if worst_position >= radius {
            // println!("Full");
            let projected = self
                .model
                .vertices
                .iter()
                .map(|x| homogenize!(m * x))
                .collect::<Vec<_>>();
            Triangles::Full {
                instance: self,
                projected,
                next_idx: 0,
            }
        } else {
            // println!("Partial");
            let projected = self
                .model
                .vertices
                .iter()
                .map(|x| m * x)
                .collect::<Vec<_>>();
            Triangles::Partial {
                instance: self,
                projected,
                next_idx: 0,
                clipping_planes: clipping_planes,
                viewport_data: (v_w, v_h, d, c_w, c_h),
                temp: vec![],
            }
        }
    }
}

/// A triangle on the canvas.
/// Contains 3 points with inverse depths and a color.
type Triangle = (
    (i64, i64, f64),
    (i64, i64, f64),
    (i64, i64, f64),
    (f64, f64, f64),
);

/// Iterator over triangles of an instance.
/// Variant depends on whether the instance is clipped fully, partially, or not.
pub enum Triangles<'a> {
    Empty,
    Partial {
        instance: &'a Instance<'a>,
        projected: Vec<Vector4<f64>>,
        next_idx: usize,
        clipping_planes: Vec<(Vector3<f64>, f64)>,
        viewport_data: (f64, f64, f64, f64, f64),
        temp: Vec<Triangle>,
    },
    Full {
        instance: &'a Instance<'a>,
        projected: Vec<(i64, i64, f64)>,
        next_idx: usize,
    },
}

impl<'a> Iterator for Triangles<'a> {
    type Item = Triangle;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            // instance is fully clipped => display nothing
            Self::Empty => None,
            // instance is partially clipped => clip each triangle separately
            Self::Partial {
                instance,
                projected,
                next_idx,
                clipping_planes,
                viewport_data: v,
                temp,
            } => {
                match temp.pop() {
                    // some temporary triangle existed => return it
                    Some(x) => Some(x),
                    // no temporary triangle existed => try generating a new one
                    None => {
                        if *next_idx >= instance.model.triangles.len() {
                            // all triangles of the model were already visited => finish returning
                            None
                        } else {
                            // some triangles remain => generate next one
                            let prev_idx = *next_idx;
                            let mut tris = {
                                let (a, b, c) = instance.model.triangles[prev_idx];
                                let (a, b, c) = (projected[a], projected[b], projected[c]);
                                vec![(a, b, c, instance.model.triangle_colors[prev_idx])]
                            };
                            // clip triangle candidates using the clipping planes
                            for (p, d) in clipping_planes {
                                let mut new_tris = vec![];
                                macro_rules! point_dot {
                                    ($a: expr) => {
                                        (Point3::from_homogeneous($a).unwrap() - Point3::origin())
                                            .dot(p)
                                    };
                                }
                                for (v1, v2, v3, col) in tris {
                                    let f1 = point_dot!(v1) >= *d;
                                    let f2 = point_dot!(v2) >= *d;
                                    let f3 = point_dot!(v3) >= *d;
                                    // count the number of vertices in front of the clipping plane
                                    let count = (f1 as usize) + (f2 as usize) + (f3 as usize);
                                    match count {
                                        0 => {
                                            // all vertices behind the clipping plane
                                            // => throw away the triangle
                                        }
                                        1 => {
                                            // one vertex in front of the clipping plane
                                            // => determine which, calculate other 2 vertices
                                            let (front, back1, back2) = match (f1, f2) {
                                                (true, _) => (v1, v2, v3),
                                                (_, true) => (v2, v3, v1),
                                                _ => (v3, v1, v2),
                                            };
                                            let mid1 = {
                                                let delta = Point3::from_homogeneous(back1)
                                                    .unwrap()
                                                    - Point3::from_homogeneous(front).unwrap();
                                                let t = (*d - point_dot!(front)) / delta.dot(p);
                                                front + t * delta.to_homogeneous()
                                            };
                                            let mid2 = {
                                                let delta = Point3::from_homogeneous(back2)
                                                    .unwrap()
                                                    - Point3::from_homogeneous(front).unwrap();
                                                let t = (*d - point_dot!(front)) / delta.dot(p);
                                                front + t * delta.to_homogeneous()
                                            };
                                            new_tris.push((front, mid1, mid2, col));
                                        }
                                        2 => {
                                            // two vertices in front of the clipping plane
                                            // => determine which, calculate 2 new vertices,
                                            //    split into 2 triangles
                                            let (back, front1, front2) = match (f1, f2) {
                                                (false, _) => (v1, v2, v3),
                                                (_, false) => (v2, v3, v1),
                                                _ => (v3, v1, v2),
                                            };
                                            let mid1 = {
                                                let delta = Point3::from_homogeneous(back).unwrap()
                                                    - Point3::from_homogeneous(front1).unwrap();
                                                let t = (*d - point_dot!(front1)) / delta.dot(p);
                                                front1 + t * delta.to_homogeneous()
                                            };
                                            let mid2 = {
                                                let delta = Point3::from_homogeneous(back).unwrap()
                                                    - Point3::from_homogeneous(front2).unwrap();
                                                let t = (*d - point_dot!(front2)) / delta.dot(p);
                                                front2 + t * delta.to_homogeneous()
                                            };
                                            new_tris.push((mid1, front1, mid2, col));
                                            new_tris.push((front1, front2, mid2, col));
                                        }
                                        _ => {
                                            // all vertices in front of the clipping plance
                                            // keep the triangle as is
                                            new_tris.push((v1, v2, v3, col));
                                        }
                                    };
                                }
                                tris = new_tris;
                            }
                            *next_idx += 1;
                            // enqueue the generated triangles to be returned
                            temp.extend(tris.into_iter().map(|(v1, v2, v3, col)| {
                                (
                                    homogeneous_3d_to_canvas(v.0, v.1, v.2, v.3, v.4, v1),
                                    homogeneous_3d_to_canvas(v.0, v.1, v.2, v.3, v.4, v2),
                                    homogeneous_3d_to_canvas(v.0, v.1, v.2, v.3, v.4, v3),
                                    col,
                                )
                            }));
                            self.next()
                        }
                    }
                }
            }
            // instance is not clipped => display all triangles
            Self::Full {
                instance,
                projected,
                next_idx,
            } => {
                if *next_idx >= instance.model.triangles.len() {
                    // all triangles of the model were already visited => finish returning
                    None
                } else {
                    // some triangles remain => generate next one
                    let prev_idx = *next_idx;
                    let (t0, t1, t2) = instance.model.triangles[prev_idx];
                    *next_idx += 1;
                    Some((
                        projected[t0],
                        projected[t1],
                        projected[t2],
                        instance.model.triangle_colors[prev_idx],
                    ))
                }
            }
        }
    }
}

/// A simple scene with three cubes.
pub fn simple_scene<'a>() -> Vec<Instance<'a>> {
    vec![
        Instance::new(
            &CUBE,
            Vector3::new(0.5, 0.5, 0.5),
            Matrix4::identity(),
            Vector3::new(-0.5, -0.5, 2.5),
        ),
        Instance::new(
            &CUBE,
            Vector3::new(0.5, 0.5, 0.5),
            Matrix4::identity(),
            Vector3::new(0.5, 1.5, 4.5),
        ),
        Instance::new(
            &CUBE,
            Vector3::new(0.5, 0.5, 0.5),
            Matrix4::identity(),
            Vector3::new(1.5, 0.5, 4.5),
        ),
    ]
}

/// A scene with a grid of `subdiv` by `subdiv` cubes.
pub fn cube_grid<'a>(subdivs: i32) -> Vec<Instance<'a>> {
    (0..2 * subdivs)
        .map(|i| {
            (0..2 * subdivs).map(move |j| {
                Instance::new(
                    &CUBE,
                    Vector3::new(0.5, 0.5, 0.5),
                    Matrix4::identity(),
                    Vector3::new(
                        (2 * (i - subdivs) + 1) as f64 * 0.5,
                        (2 * (j - subdivs) + 1) as f64 * 0.5,
                        (2 * subdivs) as f64 + 0.5,
                    ),
                )
            })
        })
        .flatten()
        .collect()
}

/// A scene with a checkerboard grid of `subdiv` by `subdiv` cubes.
pub fn cube_checkerboard<'a>(subdivs: i32) -> Vec<Instance<'a>> {
    (0..2 * subdivs)
        .map(|i| {
            (0..2 * subdivs).map(move |j| {
                if i % 2 == j % 2 {
                    Some(Instance::new(
                        &CUBE,
                        Vector3::new(0.5, 0.5, 0.5),
                        Matrix4::identity(),
                        Vector3::new(
                            (2 * (i - subdivs) + 1) as f64 * 0.5,
                            (2 * (j - subdivs) + 1) as f64 * 0.5,
                            (2 * subdivs) as f64 + 0.5,
                        ),
                    ))
                } else {
                    None
                }
            })
        })
        .flatten()
        .flatten()
        .collect()
}

/// Determine whether the camera is looking at the front or at the back of the triangle.
pub fn cull_triangle(p: &(i64, i64, f64), q: &(i64, i64, f64), r: &(i64, i64, f64)) -> bool {
    let m = Matrix3::new(
        1 as f64, p.0 as f64, p.1 as f64, 1 as f64, q.0 as f64, q.1 as f64, 1 as f64, r.0 as f64,
        r.1 as f64,
    );
    m.determinant() > 0.
}

pub struct Camera {
    pub translation: Vector3<f64>,
    pub rotation: Matrix4<f64>,
    pub perspective: Vector3<f64>,
}

impl Camera {
    pub fn default() -> Camera {
        Camera {
            translation: Vector3::new(0., 0., 0.),
            rotation: Matrix4::identity(),
            perspective: Vector3::new(1., 1., 1.),
        }
    }

    pub fn transform(&self) -> Matrix4<f64> {
        homogeneous_translation_vector(self.translation) * self.rotation
    }

    pub fn inverse_transform(&self) -> Matrix4<f64> {
        (homogeneous_translation_vector(self.translation) * self.rotation)
            .try_inverse()
            .unwrap()
    }
}
