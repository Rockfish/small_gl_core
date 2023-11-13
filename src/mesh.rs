use crate::shader::Shader;
use crate::texture::Texture;
use glad_gl::gl;
use glad_gl::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use glam::{vec3, Mat4, Vec2, Vec3};
use std::mem;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, packed)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn white() -> Self {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Vertex {
    pub position: Vec3,
    pub tex_coords: Vec2,
    pub color: Color,
}

impl Vertex {
    pub fn new(position: Vec3, tex_coords: Vec2, color: Color) -> Vertex {
        Vertex {
            position,
            tex_coords,
            color,
        }
    }
}
impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Default::default(),
            tex_coords: Default::default(),
            color: Color::white(),
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub texture: Rc<Texture>,
    pub vao: u32,
    pub vbo: u32,
    pub ebo: u32,
    pub flip_to_xz: bool,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, texture: &Rc<Texture>, flip_to_xz: bool) -> Mesh {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            // load vertex data into vertex buffers
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<Vertex>()) as GLsizeiptr,
                vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // load index data into element buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * mem::size_of::<u32>()) as GLsizeiptr,
                indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // vertex positions
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, mem::size_of::<Vertex>() as GLsizei, 0 as *const GLvoid);

            // vertex texture coordinates
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, tex_coords) as *const GLvoid,
            );

            // vertex color
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, color) as *const GLvoid,
            );

            gl::BindVertexArray(0);
        }

        Mesh {
            vertices,
            indices,
            texture: texture.clone(),
            vao,
            vbo,
            ebo,
            flip_to_xz,
        }
    }

    pub fn render(&self, shader: &Shader, position: Vec3, angle: f32, scale: Vec3) {
        let position = if self.flip_to_xz {
            vec3(position.x - 400.0, 0.0, position.y - 400.0)
        } else {
            position
        };

        let mut model_transform = Mat4::from_translation(position);

        if self.flip_to_xz {
            model_transform *= Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), 90f32.to_radians());
        }

        model_transform *= Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), angle.to_radians());
        model_transform *= Mat4::from_scale(scale);
        shader.setMat4("model", &model_transform);

        let texture_location = 0;
        shader.setInt("texture_diffuse1", texture_location as i32);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + texture_location);
            gl::BindVertexArray(self.vao);
            gl::BindTexture(gl::TEXTURE_2D, self.texture.id);
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, 0 as *const GLvoid);
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}
