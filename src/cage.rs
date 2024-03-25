use glam::Vec3;


// Faces should be rendered without any backface culling 
const FACE_DRAW_INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];


/// Cage is set of quads
pub struct Cage {
	pub vertices: Vec<Vec3>, 
	pub faces: Vec<[u32; 4]>, 
}

