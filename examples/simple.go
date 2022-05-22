package main

import (
	isosurface "github.com/Yeicor/sdfx-isosurface"
	"github.com/deadsy/sdfx/render"
	"github.com/deadsy/sdfx/render/dc"
	. "github.com/deadsy/sdfx/sdf"
	"log"
	"time"
)

func scene() SDF3 {
	b, _ := Box3D(V3{X: 1, Y: 1, Z: 1}, 0)
	s, _ := Sphere3D(0.6)
	return Difference3D(b, s)
}

func main() {
	s := scene()
	t1 := time.Now()
	render.ToSTL(s, 32, "examples/simple1.stl", dc.NewDualContouringDefault())
	t2 := time.Now()
	render.ToSTL(s, 32, "examples/simple2.stl", isosurface.NewRendererCompatible())
	td2 := time.Since(t2)
	td1 := t2.Sub(t1)
	log.Println("DualContouring delta time:", td1, "- OUR DualContouring delta time:", td2)
}
