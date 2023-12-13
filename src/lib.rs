#![feature(offset_of)]

#[allow(clippy::all)]
pub mod gl;

pub mod animator;
pub mod assimp_dump;
pub mod camera;
pub mod error;
pub mod hash_map;
pub mod macros;
pub mod mesh;
pub mod model;
pub mod model_animation;
pub mod model_mesh;
pub mod node_animation;
pub mod shader;
pub mod sprite_model;
pub mod texture;
pub mod transform;
pub mod utils;

type ShaderId = u32;

pub const SIZE_OF_FLOAT: usize = std::mem::size_of::<f32>();
