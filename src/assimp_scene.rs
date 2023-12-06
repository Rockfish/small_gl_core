#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_variables)]

use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::os::raw::c_uint;

use crate::error::Error;
use crate::error::Error::ModelError;
use crate::texture::TextureType;
use russimp::scene::*;
use russimp::sys::*;
use russimp::*;

pub const aiTextureType_NONE: c_uint = 0x0;
pub const aiTextureType_DIFFUSE: c_uint = 0x1;
pub const aiTextureType_SPECULAR: c_uint = 0x2;
pub const aiTextureType_AMBIENT: c_uint = 0x3;
pub const aiTextureType_EMISSIVE: c_uint = 0x4;
pub const aiTextureType_HEIGHT: c_uint = 0x5;
pub const aiTextureType_NORMALS: c_uint = 0x6;
pub const aiTextureType_SHININESS: c_uint = 0x7;
pub const aiTextureType_OPACITY: c_uint = 0x8;
pub const aiTextureType_DISPLACEMENT: c_uint = 0x9;
pub const aiTextureType_LIGHTMAP: c_uint = 0xA;
pub const aiTextureType_REFLECTION: c_uint = 0xB;
pub const aiTextureType_UNKNOWN: c_uint = 0xC;

type aiTextureType = u32;

// This is just a lightweight wrapper around aiScene
#[derive(Debug)]
pub struct AssimpScene {
    pub assimp_scene: *const aiScene,
}

impl AssimpScene {
    pub fn load_assimp_scene(file_path: impl Into<String>) -> Result<AssimpScene, Error> {
        let scene = AssimpScene::from_file(
            file_path.into(),
            vec![
                PostProcess::Triangulate,
                PostProcess::GenerateSmoothNormals,
                PostProcess::FlipUVs,
                PostProcess::CalculateTangentSpace,
                PostProcess::FixOrRemoveInvalidData,
                // PostProcess::JoinIdenticalVertices,
                // PostProcess::SortByPrimitiveType,
                // PostProcess::EmbedTextures,
            ],
        );

        let assimp_scene = scene?;
        Ok(assimp_scene)
    }

    pub fn from_file(
        file_path: String,
        flags: PostProcessSteps,
    ) -> Result<AssimpScene, RussimpError> {
        let bitwise_flag = flags.into_iter().fold(0, |acc, x| acc | (x as u32));
        let file_path = CString::new(file_path).unwrap();

        let raw_scene = AssimpScene::get_scene_from_file(file_path, bitwise_flag);

        // if raw_scene.is_some() {
        //     return Ok(AssimpScene{assimp_scene: raw_scene.unwrap()});
        // }
        // Err(AssimpScene::get_error())
        Ok(AssimpScene {
            assimp_scene: raw_scene,
        })
    }

    #[inline]
    fn get_scene_from_file<'a>(string: CString, flags: u32) -> *const aiScene {
        unsafe { aiImportFile(string.as_ptr(), flags) }
    }

    fn get_error() -> RussimpError {
        let error_buf = unsafe { aiGetErrorString() };
        let error = unsafe { CStr::from_ptr(error_buf).to_string_lossy().into_owned() };
        RussimpError::Import(error)
    }
}

impl Drop for AssimpScene {
    fn drop(&mut self) {
        unsafe {
            aiReleaseImport(self.assimp_scene);
        }
    }
}

/// # Safety
///
/// This function calls into the assimp library.
pub unsafe fn get_material_texture_filename(
    material: *mut aiMaterial,
    texture_type: TextureType,
    index: u32,
) -> Result<String, Error> {
    let mut path = MaybeUninit::uninit();
    let mut texture_mapping = MaybeUninit::uninit();
    let mut uv_index = MaybeUninit::uninit();
    let mut blend = MaybeUninit::uninit();
    let mut op = MaybeUninit::uninit();
    let mut map_mode: [u32; 2] = [0, 0];

    let mut flags = MaybeUninit::uninit();

    let result = aiGetMaterialTexture(
        material,
        texture_type.into(),
        index,
        path.as_mut_ptr(),
        texture_mapping.as_mut_ptr(),
        uv_index.as_mut_ptr(),
        blend.as_mut_ptr(),
        op.as_mut_ptr(),
        map_mode.as_mut_ptr() as *mut _,
        flags.as_mut_ptr(),
    );

    if result == aiReturn_aiReturn_SUCCESS {
        let filename: String = unsafe { path.assume_init() }.into();
        return Ok(filename);
    }
    Err(Error::TextureError("aiGetMaterialTexture Error: Texture not found".to_string()))
}

impl From<TextureType> for aiTextureType {
    fn from(value: TextureType) -> Self {
        match value {
            TextureType::None => 0,
            TextureType::Diffuse => 1,
            TextureType::Specular => 2,
            TextureType::Ambient => 3,
            TextureType::Emissive => 4,
            TextureType::Height => 5,
            TextureType::Normals => 6,
            TextureType::Shininess => 7,
            TextureType::Opacity => 8,
            TextureType::Displacement => 9,
            TextureType::Lightmap => 10,
            TextureType::Reflection => 11,
            TextureType::BaseColor => 12,
            TextureType::NormalCamera => 13,
            TextureType::EmissionColor => 14,
            TextureType::Metalness => 15,
            TextureType::Roughness => 16,
            TextureType::AmbientOcclusion => 17,
            TextureType::Unknown => 18,
            TextureType::Sheen => 19,
            TextureType::ClearCoat => 20,
            TextureType::Transmission => 21,
            TextureType::Force32bit => 2147483647,
        }
    }
}
