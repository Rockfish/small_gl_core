use std::collections::HashMap;
use crate::gl;
use crate::gl::{GLsizei, GLsizeiptr, GLvoid};
use crate::shader::Shader;
use crate::texture::{Texture, TextureType};
use glam::u32;
use glam::*;
use std::mem;
use std::ops::Add;
use std::rc::Rc;

const MAX_BONE_INFLUENCE: usize = 4;
const OFFSET_OF_NORMAL: usize = mem::offset_of!(ModelVertex, normal);
const OFFSET_OF_TEXCOORDS: usize = mem::offset_of!(ModelVertex, uv);
const OFFSET_OF_TANGENT: usize = mem::offset_of!(ModelVertex, tangent);
const OFFSET_OF_BITANGENT: usize = mem::offset_of!(ModelVertex, bi_tangent);
const OFFSET_OF_BONE_IDS: usize = mem::offset_of!(ModelVertex, bone_ids);
const OFFSET_OF_WEIGHTS: usize = mem::offset_of!(ModelVertex, bone_weights);

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct ModelVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub tangent: Vec3,
    pub bi_tangent: Vec3,
    pub bone_ids: [i32; MAX_BONE_INFLUENCE],
    pub bone_weights: [f32; MAX_BONE_INFLUENCE],
}

impl ModelVertex {
    pub fn new() -> ModelVertex {
        ModelVertex {
            position: Vec3::default(),
            normal: Vec3::default(),
            uv: Vec2::default(),
            tangent: Vec3::default(),
            bi_tangent: Vec3::default(),
            bone_ids: [-1; MAX_BONE_INFLUENCE],
            bone_weights: [0.0; MAX_BONE_INFLUENCE],
        }
    }

    pub fn set_bone_data_to_default(&mut self) {
        for i in 0..MAX_BONE_INFLUENCE {
            self.bone_ids[i] = -1;
            self.bone_weights[i] = 0.0;
        }
    }

    pub fn set_bone_data(&mut self, bone_id: i32, weight: f32) {
        //set first available free spot if there is any
        for i in 0..MAX_BONE_INFLUENCE {
            if self.bone_ids[i] < 0 {
                self.bone_ids[i] = bone_id;
                self.bone_weights[i] = weight;
                break;
            }
        }
    }
}

impl Default for ModelVertex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ModelMesh {
    pub name: String,
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Rc<Texture>>,
    pub node_transform: Mat4,
    pub vao: u32,
    pub vbo: u32,
    pub ebo: u32,
}

impl ModelMesh {
    pub fn new(
        name: impl Into<String>,
        vertices: Vec<ModelVertex>,
        indices: Vec<u32>,
        textures: Vec<Rc<Texture>>,
    ) -> ModelMesh {
        let mut mesh = ModelMesh {
            name: name.into(),
            vertices,
            indices,
            textures,
            node_transform: Mat4::IDENTITY,
            vao: 0,
            vbo: 0,
            ebo: 0,
        };
        mesh.setup_mesh();
        mesh
    }

    // pub fn render(&self, shader: &Rc<Shader>) {
    //     unsafe {
    //         // set the location and binding for all the textures
    //         for (texture_count, texture_sample) in self.textures.iter().enumerate() {
    //             // active proper texture unit before binding
    //             gl::ActiveTexture(gl::TEXTURE0 + texture_count as u32);
    //             // bind the texture to the texture unit
    //             gl::BindTexture(gl::TEXTURE_2D, texture_sample.texture.id);
    //
    //             shader.set_int(&texture_sample.sample_name, texture_count as i32);
    //         }
    //
    //         gl::BindVertexArray(self.vao);
    //         gl::DrawElements(
    //             gl::TRIANGLES,
    //             self.indices.len() as i32,
    //             gl::UNSIGNED_INT,
    //             std::ptr::null::<GLvoid>(),
    //         );
    //         gl::BindVertexArray(0);
    //     }
    // }
    pub fn render(&self, shader: &Rc<Shader>) {

        let mut texture_count_map: HashMap<TextureType, u32> = HashMap::new();

        unsafe {
            for (texture_unit, texture) in self.textures.iter().enumerate() {

                if !texture_count_map.contains_key(&texture.texture_type) {
                    texture_count_map.insert(texture.texture_type.clone(), 0);
                }

                let num = texture_count_map[&texture.texture_type] + 1;
                *texture_count_map.get_mut(&texture.texture_type).unwrap() = num;

                let texture_name = texture
                    .texture_type
                    .to_string()
                    .add(&num.to_string());

                shader.set_int(&texture_name, texture_unit as i32);

                gl::ActiveTexture(gl::TEXTURE0 + texture_unit as u32);
                gl::BindTexture(gl::TEXTURE_2D, texture.id);
            }

            shader.set_mat4("nodeTransform", &self.node_transform);

            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null::<GLvoid>(),
            );
            gl::BindVertexArray(0);
        }
    }

    fn setup_mesh(&mut self) {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::GenBuffers(1, &mut self.vbo);
            gl::GenBuffers(1, &mut self.ebo);

            // load vertex data into vertex buffers
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * mem::size_of::<ModelVertex>()) as GLsizeiptr,
                self.vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // load index data into element buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * mem::size_of::<u32>()) as GLsizeiptr,
                self.indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // set the vertex attribute pointers vertex Positions
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                std::ptr::null::<GLvoid>(),
            );

            // vertex normals
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                (OFFSET_OF_NORMAL) as *const GLvoid,
            );

            // vertex texture coordinates
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                (OFFSET_OF_TEXCOORDS) as *const GLvoid,
            );

            // vertex tangent
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(
                3,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                (OFFSET_OF_TANGENT) as *const GLvoid,
            );

            // vertex bitangent
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(
                4,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                (OFFSET_OF_BITANGENT) as *const GLvoid,
            );

            // bone ids
            gl::EnableVertexAttribArray(5);
            gl::VertexAttribIPointer(
                5,
                4,
                gl::INT,
                mem::size_of::<ModelVertex>() as GLsizei,
                (OFFSET_OF_BONE_IDS) as *const GLvoid,
            );

            // weights
            gl::EnableVertexAttribArray(6);
            gl::VertexAttribPointer(
                6,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                (OFFSET_OF_WEIGHTS) as *const GLvoid,
            );

            gl::BindVertexArray(0);
        }
    }
}

impl Drop for ModelMesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}

pub fn print_model_mesh(mesh: &ModelMesh) {
    println!("mesh: {:#?}", mesh);

    println!("size vertex: {}", mem::size_of::<ModelVertex>());
    println!("OFFSET_OF_NORMAL: {}", mem::offset_of!(ModelVertex, normal));
    println!("OFFSET_OF_TEXCOORDS: {}", mem::offset_of!(ModelVertex, uv));
    println!(
        "OFFSET_OF_TANGENT: {}",
        mem::offset_of!(ModelVertex, tangent)
    );
    println!(
        "OFFSET_OF_BITANGENT: {}",
        mem::offset_of!(ModelVertex, bi_tangent)
    );
    println!(
        "OFFSET_OF_BONE_IDS: {}",
        mem::offset_of!(ModelVertex, bone_ids)
    );
    println!(
        "OFFSET_OF_WEIGHTS: {}",
        mem::offset_of!(ModelVertex, bone_weights)
    );

    println!("size of Vec3: {}", mem::size_of::<Vec3>());
    println!("size of Vec2: {}", mem::size_of::<Vec2>());
    println!(
        "size of [i32;4]: {}",
        mem::size_of::<[i32; MAX_BONE_INFLUENCE]>()
    );
    println!(
        "size of [f32;4]: {}",
        mem::size_of::<[f32; MAX_BONE_INFLUENCE]>()
    );

    println!(
        "size of vertex parts: {}",
        mem::size_of::<Vec3>() * 4
            + mem::size_of::<Vec2>()
            + mem::size_of::<[i32; MAX_BONE_INFLUENCE]>()
            + mem::size_of::<[f32; MAX_BONE_INFLUENCE]>()
    );
}
