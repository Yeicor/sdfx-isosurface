use std::fmt::Debug;
use std::mem;

use implicit3d::{BoundingBox, Object, ObjectClone};
use isosurface::DualContouring;
use isosurface::extractor::IndexedVertices;
use isosurface::feature::MinimiseQEF;
use isosurface::math::Vec3;
use isosurface::sampler::Sampler;
use nalgebra::{Point3, Vector3};

pub use crate::implicit::Implicit3DObj;

pub mod implicit;

extern "C" {
    /// receives the mesh after calling mesh(), takes ownership of the vector (and should free it).
    /// Returns:
    /// - the array (pointer + length) of vertex floats, in XYZ order.
    /// - the array (pointer + length) of triangles vertex indices.
    pub fn sdf_mesh_receiver(vertices_ptr: *mut f32, vertices_len: usize,
                             indices_ptr: *mut u32, indices_len: usize);
}

/// Generates a mesh, using the imports sdf_aabb and sdf_eval to query the SDF.
#[no_mangle]
pub extern "C" fn mesh(mesh_cells: u32) {
    let mut sdf = Implicit3DObj::new(Box::new(ExternSDF::default()));
    let scale = 1.0;
    sdf.scale_0n(scale);
    let sampler = Sampler::new(&sdf);

    let mut vertices = vec![];
    let mut indices = vec![];
    let mut extractor = IndexedVertices::new(&mut vertices, &mut indices);

    // let max_level = 5;
    // let grid_size = 2usize.pow(max_level as u32);
    let grid_size = next_power_of_two(mesh_cells) as usize;
    // let mut alg = MarchingCubes::<Signed>::new(grid_size);
    // let mut alg = LinearHashedMarchingCubes::new((grid_size as f64).log2() as usize);
    // let mut alg = ExtendedMarchingCubes::new(grid_size);
    let mut alg = DualContouring::new(grid_size, MinimiseQEF {});
    // let mut alg = DualContouring::new(grid_size, ParticleBasedMinimisation {});
    alg.extract(&sampler, &mut extractor);

    // Scale back the returned vertices
    for vert_coords in vertices.chunks_exact_mut(3) {
        let v = Vec3::new(vert_coords[0], vert_coords[1], vert_coords[2]);
        let v = sdf.scale_back(v, scale);
        (*vert_coords)[0] = v.x;
        (*vert_coords)[1] = v.y;
        (*vert_coords)[2] = v.z;
    }

    // Get pointers to send back to the app
    vertices.shrink_to_fit();
    assert_eq!(vertices.len(), vertices.capacity());
    let vertices_ptr = vertices.as_mut_ptr();
    let vertices_len = vertices.len();
    mem::forget(vertices); // prevent deallocation in Rust
    indices.shrink_to_fit();
    assert_eq!(indices.len(), indices.capacity());
    let indices_ptr = indices.as_mut_ptr();
    let indices_len = indices.len();
    mem::forget(indices); // prevent deallocation in Rust

    // Notify the app of the results
    unsafe {
        sdf_mesh_receiver(vertices_ptr, vertices_len, indices_ptr, indices_len);
    }
}

fn next_power_of_two(mut mesh_cells: u32) -> u32 {
    mesh_cells -= 1;
    mesh_cells |= mesh_cells >> 1;
    mesh_cells |= mesh_cells >> 2;
    mesh_cells |= mesh_cells >> 4;
    mesh_cells |= mesh_cells >> 8;
    mesh_cells |= mesh_cells >> 16;
    mesh_cells += 1;
    mesh_cells
}

extern "C" {
    /// returns the bounding box that contains the SDF (minimum and maximum 3D points)
    pub fn sdf_aabb() -> &'static [[f32; 3]; 2];
    /// returns the signed distance to the surface in the given 3D point
    pub fn sdf_eval(point: &[f32; 3]) -> f32;
}

#[derive(Debug)]
struct ExternSDF {
    bb: BoundingBox<f32>,
}

impl Default for ExternSDF {
    fn default() -> Self {
        let bb = unsafe { sdf_aabb() };
        let bb2 = BoundingBox {
            min: Point3::new(bb[0][0], bb[0][1], bb[0][2]),
            max: Point3::new(bb[1][0], bb[1][1], bb[1][2]),
        };
        Self { bb: bb2 }
    }
}

impl ObjectClone<f32> for ExternSDF {
    fn clone_box(&self) -> Box<dyn Object<f32>> {
        Box::new(ExternSDF::default())
    }
}

impl Object<f32> for ExternSDF {
    fn bbox(&self) -> &BoundingBox<f32> {
        &self.bb
    }

    fn approx_value(&self, p: &Point3<f32>, _ign: f32) -> f32 {
        unsafe { sdf_eval(&[p.x as f32, p.y as f32, p.z as f32]) }
    }

    fn normal(&self, p: &Point3<f32>) -> Vector3<f32> {
        const EPS: f32 = 0.0001;
        Vector3::new(
            self.approx_value(&Point3::new(p.x + EPS, p.y, p.z), 0.)
                - self.approx_value(&Point3::new(p.x - EPS, p.y, p.z), 0.),
            self.approx_value(&Point3::new(p.x, p.y + EPS, p.z), 0.)
                - self.approx_value(&Point3::new(p.x, p.y - EPS, p.z), 0.),
            self.approx_value(&Point3::new(p.x, p.y, p.z + EPS), 0.)
                - self.approx_value(&Point3::new(p.x, p.y, p.z - EPS), 0.),
        )
            .normalize()
    }
}
