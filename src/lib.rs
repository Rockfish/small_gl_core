#![feature(offset_of)]

pub mod assimp_scene;
pub mod camera;
pub mod error;

#[allow(clippy::all)]
pub mod gl;

pub mod macros;
pub mod mesh;

pub mod model;

pub mod model_animation;
pub mod animator;
pub mod bone_data;
pub mod model_mesh;
pub mod shader;
pub mod sprite_model;
pub mod texture;
mod assimp_dump;
mod assimp_utils;

type ShaderId = u32;

pub const SIZE_OF_FLOAT: usize = std::mem::size_of::<f32>();
