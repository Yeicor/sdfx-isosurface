use implicit3d::{Object, PlaneY, Sphere};
use isosurface::distance::{Directed, Signed};
use isosurface::math::Vec3;
use isosurface::source::{HermiteSource, ScalarSource, VectorSource};
use nalgebra::{Point3, Vector3};

pub struct Implicit3DObj {
    pub sdf: Box<dyn Object<f32>>,
}

impl Default for Implicit3DObj {
    fn default() -> Self {
        let sdf = implicit3d::Intersection::difference_from_vec(
            vec![Box::new(Sphere::new(1.0)), Box::new(PlaneY::new(-0.5))],
            0.0,
        ).unwrap();
        let slf = Self { sdf };
        slf
    }
}

impl Implicit3DObj {
    pub fn new(sdf: Box<dyn Object<f32>>) -> Self {
        Self { sdf }
    }
    /// Move SDF to [0, N] on all axes
    pub fn scale_0n(&mut self, n: f32) {
        let bb = self.sdf.bbox();
        let extra = 0.1;
        let mut bb_size = bb.max.clone() - bb.min.clone();
        bb_size /= n;
        let bb_min = bb.min.clone() - bb_size * (extra / 2.);
        bb_size *= 1. + extra;
        self.sdf = self
            .sdf
            .clone_box()
            .translate(&Vector3::new(-bb_min.x, -bb_min.y, -bb_min.z))
            .scale(&Vector3::new(
                1. / bb_size.x,
                1. / bb_size.y,
                1. / bb_size.z,
            ));
    }
    pub fn scale_back(&self, p: Vec3, n: f32) -> Vec3 { // FIXME: There is a bug that cuts the mesh
        let bb = self.sdf.bbox();
        let extra = 0.1;
        let mut bb_size = bb.max.clone() - bb.min.clone();
        bb_size /= n;
        let bb_min = bb.min.clone() - bb_size * (extra / 2.);
        bb_size *= 1. + extra;
        Vec3::new(p.x * bb_size.x + bb_min.x, p.y * bb_size.y + bb_min.y, p.z * bb_size.z + bb_min.z)
    }
    fn vec3_to_sample_point(&self, p: Vec3) -> Point3<f32> {
        Point3::new(p.x as f32, p.y as f32, p.z as f32)
    }
    fn to_res(&self, res: Vector3<f32>) -> Vec3 {
        Vec3::new(res.x as f32, res.y as f32, res.z as f32)
    }
}

impl ScalarSource for Implicit3DObj {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        Signed(-self.sdf.approx_value(&self.vec3_to_sample_point(p), 0.) as f32)
    }
}

impl VectorSource for Implicit3DObj {
    fn sample_vector(&self, p: Vec3) -> Directed {
        Directed(self.sample_normal(p) * self.sample_scalar(p).0)
    }
}

impl HermiteSource for Implicit3DObj {
    fn sample_normal(&self, p: Vec3) -> Vec3 {
        self.to_res(-self.sdf.normal(&self.vec3_to_sample_point(p)))
    }
}

pub fn main() {} // Ignore me
