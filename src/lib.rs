#![feature(offset_of)]

use glam::{Quat, Vec2, Vec3, Vec4};
use std::mem;

#[allow(clippy::all)]
pub mod gl;

pub mod animator;
pub mod camera;
pub mod error;
pub mod hash_map;
pub mod macros;
pub mod math;
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

pub const SIZE_OF_FLOAT: usize = mem::size_of::<f32>();
pub const SIZE_OF_VEC2: usize = mem::size_of::<Vec2>();
pub const SIZE_OF_VEC3: usize = mem::size_of::<Vec3>();
pub const SIZE_OF_VEC4: usize = mem::size_of::<Vec4>();
pub const SIZE_OF_QUAT: usize = mem::size_of::<Quat>();
