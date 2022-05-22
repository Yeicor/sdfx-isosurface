extern crate core;

use std::fs::File;
use std::io::Write;

use isosurface::DualContouring;
use isosurface::extractor::IndexedInterleavedNormals;
use isosurface::feature::MinimiseQEF;
use isosurface::sampler::Sampler;

pub mod implicit;

/// This is just an example to check that it works
fn main() {
    let max_level = 5;
    let grid_size = 2usize.pow(max_level as u32);

    let mut sdf = implicit::Implicit3DObj::default();
    sdf.scale_0n(1. as f32);
    let sampler = Sampler::new(&sdf);

    let mut vertices = vec![];
    let mut indices = vec![];
    let mut extractor = IndexedInterleavedNormals::new(&mut vertices, &mut indices, &sampler);

    // let mut alg = MarchingCubes::<Signed>::new(grid_size);
    // let mut alg = LinearHashedMarchingCubes::new((grid_size as f32).log2() as usize);
    // let mut alg = ExtendedMarchingCubes::new(grid_size);
    let mut alg = DualContouring::new(grid_size, MinimiseQEF {});
    // let mut alg = DualContouring::new(grid_size, ParticleBasedMinimisation {});
    alg.extract(&sampler, &mut extractor);

    println!("Vertices: {:?}\r\nIndices: {:?}", vertices, indices);
    let mut f = File::create("examples/sphere.obj").unwrap();
    f.write_all("# Vertices\r\n".as_bytes()).unwrap();
    for vert in vertices.chunks_exact(6) {
        f.write_all(format!("v {} {} {}\r\n", vert[0], vert[1], vert[2]).as_bytes())
            .unwrap();
        // let norm = sdf.sample_normal(Vec3::new(vert[0], vert[1], vert[2]));
        // f.write_all(format!("vn {} {} {}\r\n", norm[0], norm[1], norm[2]).as_bytes()).unwrap();
    }
    f.write_all("# Faces\r\n".as_bytes()).unwrap();
    for face in indices.chunks_exact(3) {
        // 1-indexed faces!
        f.write_all(format!("f {} {} {}\r\n", face[0] + 1, face[1] + 1, face[2] + 1).as_bytes())
            .unwrap();
    }
}
