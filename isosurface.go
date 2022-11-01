package sdfx_isosurface

import (
	"context"
	_ "embed"
	"github.com/deadsy/sdfx/render"
	"github.com/deadsy/sdfx/sdf"
	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/api"
	"log"
	"strconv"
)

//go:embed isosurface-api/target/wasm32-unknown-unknown/release/isosurface_api.wasm
var wasmBytes []byte

var _ render.Render3 = &Renderer{}

type Renderer struct {
	runtime wazero.Runtime // TODO: This should be closed
}

// ctx defaults until Renderer functions are context-aware.
var ctx = context.Background()

//goland:noinspection GoUnusedExportedFunction
func NewRendererFast() *Renderer {
	return &Renderer{runtime: wazero.NewRuntimeWithConfig(ctx, wazero.NewRuntimeConfigCompiler())}
}

func NewRendererCompatible() *Renderer {
	return &Renderer{runtime: wazero.NewRuntimeWithConfig(ctx, wazero.NewRuntimeConfigInterpreter())}
}

func (r *Renderer) Render(sdf3 sdf.SDF3, meshCells int, output chan<- *render.Triangle3) {
	// Prepare the imports for providing access to our SDF to the code
	h := &host{sdf3: sdf3, output: output}
	_, err := r.runtime.NewHostModuleBuilder("env").
		NewFunctionBuilder().WithFunc(h.aabb).Export("sdf_aabb").
		NewFunctionBuilder().WithFunc(h.eval).Export("sdf_eval").
		// Prepare the import for receiving the results
		NewFunctionBuilder().WithFunc(h.meshReceiver).Export("sdf_mesh_receiver").
		Instantiate(ctx, r.runtime)
	if err != nil {
		log.Panicln(err)
	}

	// Instantiate the module and return its exported functions
	module, err := r.runtime.InstantiateModuleFromBinary(ctx, wasmBytes)
	if err != nil {
		log.Panicln(err)
	}

	// Actually run the meshing algorithm (will receive output triangles in callback above)
	_, err = module.ExportedFunction("mesh").Call(ctx, uint64(meshCells))
	if err != nil {
		log.Panicln(err)
	}
}

func (r *Renderer) Info(_ sdf.SDF3, meshCells int) string {
	return "Dual Contouring renderer (meshCells: " + strconv.Itoa(meshCells) + ")"
}

type host struct {
	sdf3   sdf.SDF3
	output chan<- *render.Triangle3
}

func (h *host) aabb(ctx context.Context, m api.Module) {
	box := h.sdf3.BoundingBox()
	m.Memory().WriteFloat32Le(ctx, 0, float32(box.Min.X))
	m.Memory().WriteFloat32Le(ctx, 4, float32(box.Min.Y))
	m.Memory().WriteFloat32Le(ctx, 8, float32(box.Min.Z))
	m.Memory().WriteFloat32Le(ctx, 12, float32(box.Max.X))
	m.Memory().WriteFloat32Le(ctx, 16, float32(box.Max.Y))
	m.Memory().WriteFloat32Le(ctx, 20, float32(box.Max.Z))
}

func (h *host) eval(ctx context.Context, m api.Module, p uint32) float32 {
	x, ok := m.Memory().ReadFloat32Le(ctx, p)
	if !ok {
		log.Panicln("Read out of range of memory")
	}
	y, ok := m.Memory().ReadFloat32Le(ctx, p+4)
	if !ok {
		log.Panicln("Read out of range of memory")
	}
	z, ok := m.Memory().ReadFloat32Le(ctx, p+8)
	if !ok {
		log.Panicln("Read out of range of memory")
	}
	return float32(h.sdf3.Evaluate(sdf.V3{X: float64(x), Y: float64(y), Z: float64(z)}))
}

func (h *host) meshReceiver(ctx context.Context, m api.Module, verticesPtr, verticesLen, indicesPtr, indicesLen uint32) {
	//fmt.Println("Got solution with", verticesLen/3, "vertices and", indicesLen, "indices!")

	// Read vertices from memory
	memIndex := verticesPtr
	var vertexCoords []float32
	for i := 0; i < int(verticesLen); i++ {
		coord, ok := m.Memory().ReadFloat32Le(ctx, memIndex)
		if !ok {
			log.Panicln("Read out of range of memory")
		}
		vertexCoords = append(vertexCoords, coord)
		memIndex += 4
	}

	// Read indices from memory
	memIndex = indicesPtr
	var indices []uint32
	for i := 0; i < int(indicesLen); i++ {
		coord, ok := m.Memory().ReadUint32Le(ctx, memIndex)
		if !ok {
			log.Panicln("Read out of range of memory")
		}
		indices = append(indices, coord)
		memIndex += 4
	}

	// Give the final triangles back to the user through the channel
	//fmt.Println("[Sample] Vertices:", vertexCoords[:9], "- Indices:", indices[:3])
	for faceIndex := 0; faceIndex < len(indices); faceIndex += 3 {
		index0 := indices[faceIndex]
		index1 := indices[faceIndex+1]
		index2 := indices[faceIndex+2]
		vertex0 := sdf.V3{X: float64(vertexCoords[index0*3]), Y: float64(vertexCoords[index0*3+1]), Z: float64(vertexCoords[index0*3+2])}
		vertex1 := sdf.V3{X: float64(vertexCoords[index1*3]), Y: float64(vertexCoords[index1*3+1]), Z: float64(vertexCoords[index1*3+2])}
		vertex2 := sdf.V3{X: float64(vertexCoords[index2*3]), Y: float64(vertexCoords[index2*3+1]), Z: float64(vertexCoords[index2*3+2])}
		tri := &render.Triangle3{V: [3]sdf.V3{vertex0, vertex1, vertex2}}
		//if faceIndex < 10 {
		//	fmt.Println(" - ", tri)
		//}
		h.output <- tri
	}
}
