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
	// Holds index into data, so will be multiple of cage len
	Boundary(usize),
    Interior(usize),
	// InteriorControl(f32), // Would need to be part of the Cage data structure, maybe rename to ControlThingy
}
impl GridCell {
	pub fn uninitialized(&self) -> bool {
		match self {
			Self::Uninitialized => true,
			_ => false,
		}
	}

	pub fn mark_boundary(&mut self, data: &mut Vec<f32>, cage_len: usize) {
		match self {
			Self::Uninitialized => {
				let i = data.len();
				*self = GridCell::Boundary(i);
				data.extend((0..cage_len).map(|_| 0.0));
			},
			Self::Boundary(i) => {},
			_ => panic!(),
		}
	}

	pub fn get_boundary<'a>(&self, data: &'a mut Vec<f32>, cage_len: usize) -> &'a mut [f32] {
		match self {
			Self::Boundary(i) => &mut data[*i..*i+cage_len],
			_ => panic!(),
		}
	}
}


pub struct Grid {
    pub min: Vec3,
    pub max: Vec3,
    pub dimensions: UVec3, 
    cell_types: Vec<GridCell>, // [Gridcell size of volume cubed]
	cell_data: Vec<f32>, // Vec<[f32; number of cage points]>
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
		
		let data_size = (dimensions[0] * dimensions[1] * dimensions[2]) as usize;
		let mut cell_types = Vec::with_capacity(data_size);
		cell_types.resize(data_size, GridCell::Uninitialized);

		Self {
			min, max, dimensions, cell_types, cell_data: Vec::new(), cage, 
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
						self.cell_types[cell_index].mark_boundary(&mut self.cell_data, self.cage.vertices.len());

						let cell_data = self.cell_types[cell_index].get_boundary(&mut self.cell_data, self.cage.vertices.len());

						cell_data[i0] = barycentric[0];
						cell_data[i1] = barycentric[1];
						cell_data[i2] = barycentric[2];
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
			if self.cell_types[index].uninitialized() {
				// Mark as exerior 
				self.cell_types[index] = GridCell::Exterior;

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
		for cell in self.cell_types.iter_mut() {
			match cell {
				GridCell::Uninitialized => {
					let i = self.cell_data.len();
					self.cell_data.extend((0..self.cage.vertices.len()).map(|_| 0.0));
					*cell = GridCell::Interior(i);
				},
				_ => {},
			}
		}
	}

	// Most basic cpu smoothing
	// Should be terrible, but good for presentation
	pub fn smooth_basic(&mut self, tau: f32) {
		let mut buffer = Vec::with_capacity(self.cell_data.len());
		let [sx, sy, sz] = self.dimensions.as_ivec3().to_array();
		let positions = (0..sx).flat_map(move |x| {
			(0..sy).flat_map(move |y| {
				(0..sz).map(move |z| {
					IVec3::new(x, y, z)
				})
			})
		})
		// Idea 1: filter by cell
		// Test performance with and without this
		.filter(|&cell| {
			let index = self.index_of(cell.as_uvec3()).unwrap();
			if let GridCell::Interior(_) = self.cell_types[index] {
				true
			} else {
				false
			}
		})
		.collect::<Vec<_>>();

		loop {
			// Store previous state
			buffer.clone_from(&self.cell_data);

			let delta_accum = 0.0;
			let delta_count = 0;

			// Idea 2: chunks for rayon jobs
			// Map to (mutable reference (unsafe), position)
			for cell in positions.iter().copied() {
				let index = self.index_of(cell.as_uvec3()).unwrap();
				
				// Only if interior
				if let GridCell::Interior(_) = self.cell_types[index] {
				} else {
					continue 
				}

				let cell_data = self.cell_types[index].get_boundary(&mut self.cell_data, self.cage.vertices.len());
				// Zero value for sum
				cell_data.iter_mut().for_each(|d| *d = 0.0);

				// For each neighbour
				let mut neighbour_count = 0;
				for d in [
					IVec3::X,
					IVec3::Y,
					IVec3::Z,
					IVec3::NEG_X,
					IVec3::NEG_Y,
					IVec3::NEG_Z,
				] {
					let n = cell + d;
					// If in bounds
					if n.cmpge(IVec3::ZERO).all() && n.cmplt(self.dimensions.as_ivec3()).all() {
						let d_index = self.index_of(n.as_uvec3()).unwrap();
						let n_data = self.cell_types[d_index].get_boundary(&mut buffer, self.cage.vertices.len());

						// Add to sum
						cell_data.iter_mut().zip(n_data).for_each(|(c, n)| *c += *n);
						neighbour_count += 1;
					}
				}

				// Cell is average of neighbours
				cell_data.iter_mut().for_each(|d| *d /= neighbour_count as f32);				
			}

			let delta = delta_accum / delta_count as f32;
			if delta <= tau {
				break
			}
		}

	}

	// Rayon jobs! 
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
		u0.cross(f0), // 9 axis 
		u0.cross(f1),
		u0.cross(f2),
		u1.cross(f0),
		u1.cross(f1),
		u1.cross(f2),
		u2.cross(f0),
		u2.cross(f1),
		u2.cross(f2),
		Vec3::X, // 3 face normals for AABB
		Vec3::Y,
		Vec3::Z,
		f0.cross(f1), // 1 face normal for triangle 
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
