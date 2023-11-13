use crate::gl;
use crate::gl::{GLchar, GLint, GLuint};
use glam::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::path::Path;
use std::ptr;

use crate::ShaderId;
use crate::*;

#[derive(Debug, Clone)]
pub struct Shader {
    pub id: ShaderId,
    pub vert_file: String,
    pub frag_file: String,
    pub geom_file: Option<String>,
}

impl Shader {
    pub fn new(
        vert_file: impl Into<String>,
        frag_file: impl Into<String>,
        geom_file: Option<impl Into<String>>,
    ) -> Result<Self, String> {
        let mut shader = Shader {
            id: 0,
            vert_file: vert_file.into(),
            frag_file: frag_file.into(),
            geom_file: geom_file.map(|f| f.into()),
        };

        let mut geometry_code: String = String::default();

        let vertex_code = match read_file(&shader.vert_file) {
            Ok(content) => content,
            Err(error) => return Err(error.to_string()),
        };

        let fragment_code = match read_file(&shader.frag_file) {
            Ok(content) => content,
            Err(error) => return Err(error.to_string()),
        };

        if let Some(geometry_path) = &shader.geom_file {
            match read_file(geometry_path) {
                Ok(content) => geometry_code = content,
                Err(error) => return Err(error.to_string()),
            }
        }

        unsafe {
            // vertex shader
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let c_string = c_string!(vertex_code);
            gl::ShaderSource(vertex_shader, 1, &c_string.as_ptr(), ptr::null());
            gl::CompileShader(vertex_shader);

            check_compile_errors(vertex_shader, "VERTEX")?;

            // fragment shader
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let c_string = c_string!(fragment_code);
            gl::ShaderSource(fragment_shader, 1, &c_string.as_ptr(), ptr::null());
            gl::CompileShader(fragment_shader);

            check_compile_errors(fragment_shader, "FRAGMENT")?;

            // geometry shader
            let mut geometry_shader: GLuint = 0;
            if shader.geom_file.is_some() {
                geometry_shader = gl::CreateShader(gl::GEOMETRY_SHADER);
                let c_string = c_string!(geometry_code);
                gl::ShaderSource(geometry_shader, 1, &c_string.as_ptr(), ptr::null());
                gl::CompileShader(geometry_shader);

                check_compile_errors(geometry_shader, "GEOMETRY")?;
            }

            // shader program
            shader.id = gl::CreateProgram();
            // link the first program object
            gl::AttachShader(shader.id, vertex_shader);
            gl::AttachShader(shader.id, fragment_shader);
            if shader.geom_file.is_some() {
                gl::AttachShader(shader.id, geometry_shader);
            }
            gl::LinkProgram(shader.id);

            check_compile_errors(shader.id, "PROGRAM")?;

            // delete the shaders as they're linked into our program now and no longer necessary
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            if shader.geom_file.is_some() {
                gl::DeleteShader(geometry_shader);
            }
        }

        Ok(shader)
    }

    pub fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn use_shader_with(&self, projection: &Mat4, view: &Mat4) {
        unsafe {
            gl::UseProgram(self.id);
        }
        self.set_mat4("projection", projection);
        self.set_mat4("view", view);
    }

    // utility uniform functions
    // ------------------------------------------------------------------------
    pub fn set_bool(&self, name: &str, value: bool) {
        unsafe {
            let v = if value { 1 } else { 0 };
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform1i(location, v);
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_int(&self, name: &str, value: i32) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform1i(location, value);
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_float(&self, name: &str, value: f32) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform1f(location, value);
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_vec2(&self, name: &str, value: &Vec2) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform2fv(location, 1, value.to_array().as_ptr());
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_vec2_xy(&self, name: &str, x: f32, y: f32) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform2f(location, x, y);
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_vec3(&self, name: &str, value: &Vec3) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform3fv(location, 1, value.to_array().as_ptr());
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_vec3_xyz(&self, name: &str, x: f32, y: f32, z: f32) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform3f(location, x, y, z);
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_vec4(&self, name: &str, value: &Vec4) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform4fv(location, 1, value.to_array().as_ptr());
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_vec4_xyzw(&self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform4f(location, x, y, z, w);
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_mat2(&self, name: &str, mat: &Mat2) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::UniformMatrix2fv(location, 1, gl::FALSE, mat.to_cols_array().as_ptr());
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_mat3(&self, name: &str, mat: &Mat3) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::UniformMatrix3fv(location, 1, gl::FALSE, mat.to_cols_array().as_ptr());
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_mat4(&self, name: &str, matrix: &Mat4) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::UniformMatrix4fv(location, 1, gl::FALSE, matrix.to_cols_array().as_ptr());
        }
    }
}

fn read_file(filename: &str) -> Result<String, Error> {
    let mut content: String = Default::default();
    let mut file = File::open(Path::new(filename))?;
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn check_compile_errors(shader_id: u32, check_type: &str) -> Result<(), String> {
    unsafe {
        let mut status = gl::FALSE as GLint;

        if check_type != "PROGRAM" {
            gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut status);
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut len);
                // Subtract 1 to skip the trailing null character.
                let mut info_log = vec![0; len as usize - 1];
                gl::GetProgramInfoLog(
                    shader_id,
                    1024,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );
                return Err(String::from_utf8_lossy(&info_log).to_string());
            }
        } else {
            gl::GetProgramiv(shader_id, gl::LINK_STATUS, &mut status);
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetProgramiv(shader_id, gl::INFO_LOG_LENGTH, &mut len);
                // Subtract 1 to skip the trailing null character.
                let mut info_log = vec![0; len as usize - 1];
                gl::GetProgramInfoLog(
                    shader_id,
                    1024,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );
                let error_msg = String::from_utf8_lossy(&info_log).to_string();
                return Err(error_msg);
            }
        }
    }
    Ok(())
}
