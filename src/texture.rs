use crate::error::Error;
use crate::gl;
use crate::gl::{GLint, GLsizei, GLuint, GLvoid};
use image::ColorType;
use serde::{Deserialize, Serialize};
use std::ffi::{c_uint, OsString};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Copy, Clone)]
pub enum TextureFilter {
    Linear,
    Nearest,
}

#[derive(Debug, Copy, Clone)]
pub enum TextureWrap {
    Clamp,
    Repeat,
}

#[derive(Debug, Clone, Copy)]
pub enum TextureType {
    None,
    Diffuse,
    Specular,
    Ambient,
    Emissive,
    Height,
    Normal,
    Shininess,
    Opacity,
    Displacement,
    Lightmap,
    Reflection,
    Unknown,
}

impl Display for TextureType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureType::Diffuse => write!(f, "texture_diffuse"),
            TextureType::Specular => write!(f, "texture_specular"),
            TextureType::Ambient => write!(f, "texture_ambient"),
            TextureType::Emissive => write!(f, "texture_emissive"),
            TextureType::Normal => write!(f, "texture_normal"),
            TextureType::Height => write!(f, "texture_height"),
            TextureType::Shininess => write!(f, "texture_shininess"),
            TextureType::Opacity => write!(f, "texture_opacity"),
            TextureType::Displacement => write!(f, "texture_displacement"),
            TextureType::Lightmap => write!(f, "texture_lightmap"),
            TextureType::Reflection => write!(f, "texture_reflection"),
            TextureType::Unknown => write!(f, "texture_unknown"),
            TextureType::None => write!(f, "texture_none"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextureConfig {
    pub texture_type: TextureType,
    pub filter: TextureFilter,
    pub wrap: TextureWrap,
    pub flip_v: bool,
    pub gamma_correction: bool,
}

impl Default for TextureConfig {
    fn default() -> Self {
        TextureConfig::new()
    }
}

impl TextureConfig {
    pub fn new() -> Self {
        TextureConfig {
            texture_type: TextureType::Diffuse,
            filter: TextureFilter::Linear,
            wrap: TextureWrap::Clamp,
            flip_v: false,
            gamma_correction: false,
        }
    }

    pub fn set_type(mut self, texture_type: TextureType) -> Self {
        self.texture_type = texture_type;
        self
    }

    pub fn set_filter(mut self, filter_type: TextureFilter) -> Self {
        self.filter = filter_type;
        self
    }

    pub fn set_wrap(mut self, wrap_type: TextureWrap) -> Self {
        self.wrap = wrap_type;
        self
    }

    pub fn set_flipv(mut self, flip_v: bool) -> Self {
        self.flip_v = flip_v;
        self
    }

    pub fn set_gamma_correction(mut self, correct_gamma: bool) -> Self {
        self.gamma_correction = correct_gamma;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub id: u32,
    pub texture_path: OsString,
    pub texture_type: TextureType,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct TextureSample {
    pub sample_name: String,
    pub texture: Rc<Texture>,
}

impl Texture {
    pub fn new(
        texture_path: impl Into<OsString>,
        texture_config: &TextureConfig,
    ) -> Result<Texture, Error> {
        let path = PathBuf::from(texture_path.into());
        let (id, width, height) = Texture::load_texture(&path, texture_config)?;
        let texture =
            Texture {
                id,
                texture_path: path.into(),
                texture_type: texture_config.texture_type,
                width,
                height,
            };
        Ok(texture)
    }

    pub fn load_texture(
        texture_path: &PathBuf,
        texture_config: &TextureConfig,
    ) -> Result<(GLuint, u32, u32), Error> {
        let mut texture_id: GLuint = 0;

        let img = image::open(texture_path)?;
        let (width, height) = (img.width() as GLsizei, img.height() as GLsizei);

        let color_type = img.color();

        let img = if texture_config.flip_v {
            img.flipv()
        } else {
            img
        };

        unsafe {
            let internal_format: c_uint;
            let data_format: c_uint;
            match color_type {
                ColorType::L8 => {
                    internal_format = gl::RED;
                    data_format = gl::RED;
                }
                ColorType::Rgb8 => {
                    internal_format = if texture_config.gamma_correction {
                        gl::SRGB
                    } else {
                        gl::RGB
                    };
                    data_format = gl::RGB;
                }
                ColorType::Rgba8 => {
                    internal_format = if texture_config.gamma_correction {
                        gl::SRGB_ALPHA
                    } else {
                        gl::RGBA
                    };
                    data_format = gl::RGBA;
                }
                _ => panic!("no mapping for color type"),
            };

            let data = match color_type {
                ColorType::L8 => img.into_rgb8().into_raw(),
                ColorType::Rgb8 => img.into_rgb8().into_raw(),
                ColorType::Rgba8 => img.into_rgba8().into_raw(),
                _ => panic!("no mapping for color type"),
            };

            gl::GenTextures(1, &mut texture_id);

            // not needed here
            // gl::ActiveTexture(gl::TEXTURE0 + texture_config.texture_unit);

            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as GLint,
                width,
                height,
                0,
                data_format,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const GLvoid,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            let wrap_param = match texture_config.wrap {
                TextureWrap::Clamp => gl::CLAMP_TO_EDGE,
                TextureWrap::Repeat => gl::REPEAT,
            };

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_param as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_param as GLint);

            match texture_config.filter {
                TextureFilter::Linear => {
                    gl::TexParameteri(
                        gl::TEXTURE_2D,
                        gl::TEXTURE_MIN_FILTER,
                        gl::LINEAR_MIPMAP_LINEAR as GLint,
                    );
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                }
                TextureFilter::Nearest => {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
                }
            }
        }
        Ok((texture_id, width as u32, height as u32))
    }
}