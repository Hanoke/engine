use ash::vk;

#[repr(C)]
pub struct Vertex {
    pub pos: glam::Vec3,
    pub uv:  glam::Vec2,
}

pub struct Model {
    pub vertices:       Vec<Vertex>,
    pub vertex_indices: Vec<u32>,
    pub rotation:       f32, 
    pub rotation_speed: f32,
    pub scale:          f32,
    pub scale_speed:    f32,
}

impl Model {
    pub fn new (model_file_path: &str) -> Model {

        let obj = obj::Obj::load(model_file_path).unwrap();
        
        let vertex_positions: Vec<glam::Vec3> = obj.data.position.iter().map(|pos| {glam::Vec3::from_array(*pos)}).collect();
        let vertex_uvs: Vec<glam::Vec2> = obj.data.texture.iter().map(|uv| {
            // ".obj" files need this operation on 'v-axis' to become compatible with vulkan.
            glam::Vec2::from_array([uv[0], 1.0f32 - uv[1]])
        }).collect();

        // TODO: Use a HashSet.
        let mut unique_index_tuples: Vec<(usize, usize)> = Vec::new();
        // Each poly has 3 index_types.
        let polys = &obj.data.objects[0].groups[0].polys;
        let triangle_count = polys.len() * 3;
        let mut vertex_indices: Vec<u32> = Vec::with_capacity(triangle_count);
        for  poly in polys {
            // Index tuple has: (vertex position index, vertex uv index, vertex normal index).
            for index_tuple in &poly.0 {
                let as_tuple = (index_tuple.0, index_tuple.1.unwrap());
                if unique_index_tuples.contains(&as_tuple) {
                    let idx = unique_index_tuples.iter().position(|elem| {*elem == as_tuple}).unwrap();
                    vertex_indices.push(idx as u32);
                } else {
                    unique_index_tuples.push(as_tuple);
                    vertex_indices.push((unique_index_tuples.len() - 1) as u32);
                }
            }
        }
        
        println!("There are {} vertex_positions.", vertex_positions.len());
        println!("There are {} vertex_uvs", vertex_uvs.len());
        println!("Found     {} unique vertices from vertex_positions and vertex_uvs.", unique_index_tuples.len());
        println!("There are {} triangles.", triangle_count);
        println!("There are {} vertex_indices.", vertex_indices.len());

        let mut vertices: Vec<Vertex> = Vec::with_capacity(unique_index_tuples.len());
        for unique_index_tuple in unique_index_tuples {
            vertices.push(Vertex { 
                pos: vertex_positions[unique_index_tuple.0],
                uv: vertex_uvs[unique_index_tuple.1]
                })
        }

        Model {
            vertices,
            vertex_indices,
            rotation: 1.2,
            rotation_speed: 0.005,
            scale: 1.0,
            scale_speed: 0.2
        }
    }
    #[inline(always)]
    pub fn get_vertex_input_binding_stride () -> u32 {
        std::mem::size_of::<Vertex>() as u32
    }
    #[inline(always)]
    pub fn get_vertex_buffer_size(&self) -> vk::DeviceSize {
        (self.vertices.len() * Model::get_vertex_input_binding_stride() as usize) as u64
    }
    #[inline(always)]
    pub fn get_index_buffer_size (&self) -> vk::DeviceSize {
        (self.vertex_indices.len() * std::mem::size_of::<u32>()) as u64
    }
}