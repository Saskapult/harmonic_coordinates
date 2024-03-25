use glam::{IVec3, Quat, UVec3, Vec3};

use crate::cage::Cage;

struct OBB {
	pub centre: Vec3,
	pub half_size: Vec3, 
	pub quat: Quat,
}
impl OBB {
	pub fn sat_collide(&self, other: &Self) -> bool {
		todo!()
	}
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

	fn depth(&self) -> usize {
		self.cage.vertices.len()
	}

	fn get_data(&self, index: usize) -> &[GridCell] {
		&self.data[index..index+self.depth()]
	}

	// This is probably the least efficent part of the implementation 
	pub fn mark_boundaries(&mut self) {
		// Boundary value will be distance to the vertex over the max distance 
		// to the vertex (for the whole quad) 

		let cell_size = (self.max - self.min) / self.dimensions.as_vec3();

		let index_of = |pos: UVec3| {
			pos[0] * self.dimensions[1] * self.dimensions[2] +
			pos[1] * self.dimensions[1] +
			pos[2]
		};

		// For each cage quad
		for indices in self.cage.faces.iter() {

			let vertices = indices
				.map(|i| self.cage.vertices[i as usize]);

			let max_distance_squared = vertices
				.map(|v| v.distance_squared(vertices[0]))
				.iter().copied().reduce(|a, v| a.max(v)).unwrap();

			// Find the plane created by this face
			let u = vertices[1] - vertices[0];
			let v = vertices[2] - vertices[0];
			let n = u.cross(v).normalize(); // maybe don't normalize yet? 
			let d = -vertices[0].dot(n);

			// Find AABB
			let aabb_min = vertices.iter().copied()
				.reduce(|a, v| a.min(v)).unwrap();
			let aabb_max = vertices.iter().copied()
				.reduce(|a, v| a.max(v)).unwrap();

			// Find cells AABB
			let cell_min = ((aabb_min - self.min) / self.dimensions.as_vec3()).floor().as_uvec3();
			let cell_max = ((aabb_max - self.min) / self.dimensions.as_vec3()).floor().as_uvec3();
			let cell_diff = cell_max - cell_min;

			// For each voxel in the plane's cells AABB
			for x in 0..cell_diff[0] {
				for y in 0..cell_diff[1] {
					for z in 0..cell_diff[2] {
						let voxel_pos = cell_min + UVec3::new(x, y, z);
						// Use AABB-plane intersection 
						// Hopefully won't go out of bounds because we 
						// constrained to the cells AABB

						// Basically copied form https://gdbooks.gitbooks.io/3dcollisions/content/Chapter2/static_aabb_plane.html
						// I take no credit
						
						let centre = voxel_pos.as_vec3() * cell_size + cell_size / 2.0; 
						let extent = voxel_pos.as_vec3() * cell_size + cell_size - centre;

						let r = extent.dot(n.abs());

						let s = n.dot(centre) - d;

						if s.abs() <= r {
							// For each intersected cell
							let index = index_of(voxel_pos);
							// For each data
							for (data_i, data) in self.data.iter_mut().enumerate() {
								// If index matches one of the quad indices, write distance to that
								// Question: How do I know the distacne to that??
								// Distance over distance to furthest edge?
								// Could look weird, but it could work
								// Let's try that for now
								if let Some(i) = indices.iter().find(|i| **i == data_i as u32) {
									let distance_squared = centre.distance_squared(vertices[*i as usize]);

									let perc = distance_squared / max_distance_squared;
									
									assert!(perc >= 0.0, "Distance percentage too small");
									assert!(perc <= 1.0, "Distance percentage too big");

									data[index as usize] = GridCell::Boundary(perc);
								} else {
									// Else write 0.0 
									data[index as usize] = GridCell::Boundary(0.0);
								}
							}
						}
					}
				}
			}
		}
	}

	pub fn fill_exterior(&mut self) {
		let mut stack = Vec::new();
		stack.push(IVec3::ZERO);

		let index_of = |pos: UVec3| {
			pos[0] * self.dimensions[1] * self.dimensions[2] +
			pos[1] * self.dimensions[1] +
			pos[2]
		};

		while let Some(pos) = stack.pop() {
			let index = index_of(pos.as_uvec3()) as usize;
			// If uninitialized, mark as exerior 
			if self.data[0][index].uninitialized() {
				for data in self.data.iter_mut() {
					data[index] = GridCell::Exterior;
				}
			}

			// Add neighbours to stack 
			for d in [IVec3::X, IVec3::Y, IVec3::Z, -IVec3::X, -IVec3::Y, -IVec3::Z] {
				let n_pos = pos + d;
				// Skip if out of bounds
				if n_pos.min_element() < 0 || n_pos.cmpge(self.dimensions.as_ivec3()).any() {
					continue
				}
				stack.push(n_pos);
			}
		}
	}

	pub fn mark_interior(&mut self) {
		for data in self.data.iter_mut() {
			for cell in data.iter_mut() {
				match cell {
					GridCell::Uninitialized => *cell = GridCell::Interior(0.0),
					_ => {},
				}
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
