use glam::{IVec3, Quat, UVec3, Vec3};


// Faces should be rendered without any backface culling 
const FACE_DRAW_INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];


/// Cage is set of quads
pub struct Cage {
	pub vertices: Vec<Vec3>, 
	pub faces: Vec<[u32; 4]>, 
}


#[derive(Debug, Clone, Copy)]
pub enum GridCell {
    Uninitialized,
    Exterior,
	Boundary(f32),
    Interior(f32),
	// InteriorControl(f32), // Would need to be part of the Cage data structure, maybe rename to ControlThingy
}
impl GridCell {
	pub fn uninitialized(&self) -> bool {
		match self {
			Self::Uninitialized => true,
			_ => false,
		}
	}
}


pub struct Grid {
    pub min: Vec3,
    pub max: Vec3,
    pub dimensions: UVec3, 
    data: Vec<GridCell>, // [[Gridcell; number of cage points]; size of volume cubed]
	pub cage: Cage
}
impl Grid {
	pub fn new(dimensions: UVec3, cage: Cage) -> Self {
		let min = cage.vertices.iter().copied()
			.reduce(|a, v| a.min(v))
			.unwrap();
		let max = cage.vertices.iter().copied()
			.reduce(|a, v| a.max(v))
			.unwrap();
		
		let data_size = (dimensions[0] * dimensions[1] * dimensions[2]) as usize * cage.vertices.len();
		let mut data = Vec::with_capacity(data_size);
		data.resize(data_size, GridCell::Uninitialized);

		Self {
			min, max, dimensions, data, cage, 
		}
	}

	#[inline]
	fn depth(&self) -> usize {
		self.cage.vertices.len()
	}

	#[inline]
	fn index_of(&self, position: UVec3) -> Option<usize> {
		position.cmplt(self.dimensions).all().then(|| 
			(position[0] * self.dimensions[0] * self.dimensions[1] * 
			position[1] * self.dimensions[1] * 
			position[3]) as usize)
	}

	#[inline]
	fn get_data(&self, index: usize) -> &[GridCell] {
		&self.data[index..index+self.depth()]
	}

	#[inline]
	fn get_data_mut(&mut self, index: usize) -> &mut [GridCell] {
		let r = index..index+self.depth();
		&mut self.data[r]
	}

	// This is probably the least efficent part of the implementation 
	pub fn mark_boundaries_simple(&mut self) {
		let cell_size = (self.max - self.min) / self.dimensions.as_vec3();

		// For each cage quad
		for quad in self.cage.faces.clone().iter() {
			for triangle in [[0, 1, 2], [2, 3, 0]] {
				let [v0, v1, v2] = triangle
					.map(|i| self.cage.vertices[quad[i] as usize]);

				let [i0, i1, i2] = triangle
					.map(|i| quad[i] as usize);

				// For non simple, take AABB of triangle and only iterate over intersection
				// Find min and max and iter over that (min..=max)
				let [sx, sy, sz] = self.dimensions.to_array();
				let positions = (0..sx).flat_map(move |x| {
					(0..sy).flat_map(move |y| {
						(0..sz).map(move |z| {
							UVec3::new(x, y, z)
						})
					})
				});
				
				for cell in positions {
					let centre = cell.as_vec3() * cell_size + cell_size / 2.0; 
					let extent = cell.as_vec3() * cell_size + cell_size - centre;
					if aabb_triangle_intersect(centre, extent, v0, v1, v2) {
						let barycentric = barycentric(centre, v0, v1, v2);

						let cell_index = self.index_of(cell).unwrap();
						let cell_data = self.get_data_mut(cell_index);

						cell_data[i0] = GridCell::Boundary(barycentric[0]);
						cell_data[i1] = GridCell::Boundary(barycentric[1]);
						cell_data[i2] = GridCell::Boundary(barycentric[2]);
					}
				}
			}
		}
	}

	pub fn fill_exterior(&mut self) {
		let mut stack = Vec::new();
		stack.push(IVec3::ZERO);

		while let Some(pos) = stack.pop() {
			let index = self.index_of(pos.as_uvec3()).unwrap();
			// If uninitialized
			if self.get_data(index).iter().all(|cell| cell.uninitialized()) {
				// Mark as exerior 
				for cell in self.get_data_mut(index) {
					*cell = GridCell::Exterior;
				}

				// Add in-bounds neighbours to stack 
				for d in [IVec3::X, IVec3::Y, IVec3::Z, -IVec3::X, -IVec3::Y, -IVec3::Z] {
					let n_pos = pos + d;
					if n_pos.min_element() < 0 || n_pos.cmpge(self.dimensions.as_ivec3()).any() {
						continue
					}
					stack.push(n_pos);
				}
			}
		}
	}

	pub fn mark_interior(&mut self) {
		for cell in self.data.iter_mut() {
			match cell {
				GridCell::Uninitialized => *cell = GridCell::Interior(0.0),
				_ => {},
			}
		}
	}

	// Most basic cpu smoothing
	// Should be terrible, but good for presentation
	pub fn smooth_basic(&mut self, tau: f32) {
		


	}

	// Raton job for each entry in data 
	pub fn smooth_threads(&mut self, tau: f32) {

	}

	// GPU is awesome 
	pub fn smooth_gpu(&mut self, tau: f32) {

	}

	
}


// https://gdbooks.gitbooks.io/3dcollisions/content/Chapter4/aabb-triangle.html
fn aabb_triangle_intersect(
	centre: Vec3,
	extent: Vec3,
	mut v0: Vec3,
	mut v1: Vec3,
	mut v2: Vec3,
) -> bool {
	v0 -= centre;
	v1 -= centre;
	v2 -= centre;

	let f0 = v1 - v0;
	let f1 = v2 - v1;
	let f2 = v0 - v2;

	let u0 = Vec3::X;
	let u1 = Vec3::Y;
	let u2 = Vec3::Z;

	
	for axis in [
		// 9 axis 
		u0.cross(f0),
		u0.cross(f1),
		u0.cross(f2),
		u1.cross(f0),
		u1.cross(f1),
		u1.cross(f2),
		u2.cross(f0),
		u2.cross(f1),
		u2.cross(f2),
		// 3 face normals for AABB
		Vec3::X,
		Vec3::Y,
		Vec3::Z,
		// 1 face normal for triangle 
		f0.cross(f1),
	] {
		let p0 = v0.dot(axis);
		let p1 = v1.dot(axis);
		let p2 = v2.dot(axis);
		let r = extent.x * u0.dot(axis).abs() +
			extent.y * u1.dot(axis).abs() +
			extent.z * u2.dot(axis).abs();
		if f32::max(-f32::max(f32::max(p0, p1), p2), f32::min(f32::min(p0, p1), p2)) > r {
			return false
		}
	}

	return true
}


// https://gamedev.stackexchange.com/questions/23743/whats-the-most-efficient-way-to-find-barycentric-coordinates
#[inline]
fn barycentric(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
	let v0 = b - a;
	let v1 = c - a;
	let v2 = p - a;
	let d00 = v0.dot(v0);
	let d01 = v0.dot(v1);
	let d11 = v1.dot(v1);
	let d20 = v2.dot(v0);
	let d21 = v2.dot(v1);
	let denom = d00 * d11 - d01 * d01;

	let v = (d11 * d20 - d01 * d21) / denom;
	let w = (d00 * d21 - d01 * d20) / denom;
	let u = 1.0 - v - w;
	Vec3::new(u, v, w)
}
