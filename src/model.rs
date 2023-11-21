use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::ops::Add;
use crate::assimp_scene::*;
use crate::error::Error;
use crate::error::Error::ModelError;
use crate::model_mesh::{ModelMesh, ModelVertex};
use crate::shader::Shader;
use crate::texture::{Texture, TextureConfig, TextureFilter, TextureSample, TextureType, TextureWrap};
use glam::*;
use russimp::scene::*;
use russimp::sys::*;
use std::os::raw::c_uint;
use std::path::PathBuf;
use std::ptr::*;
use std::rc::Rc;
use image::codecs::png::CompressionType::Default;
use russimp::animation::Animation;
use crate::animation::convert_to_mat4;
use crate::bone::BoneInfo;

// Animation
// aiVector3D => Vec3

// #[repr(u32)]
// #[derive(Debug, Clone, PartialEq)]
// pub enum AnimationBehaviour {
//     DEFAULT= 0,
//     CONSTANT= 1,
//     LINEAR= 2,
//     REPEAT= 3,
//     Force32Bit= 2147483647,
// }

// #[derive(Debug, Clone, PartialEq)]
// pub struct VectorKey {
//     pub time: f64,
//     pub value: Vec3
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct QuatKey {
//     pub time: f64,
//     pub value: Quat,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct MeshKey {
//     pub time: f64,
//     pub value: u32,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct MeshMorphKey {
//     pub time: f64,
//     pub values: Vec<u32>,
//     pub weights: Vec<f64>,
//     pub num_values_and_weights: u32,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct NodeAnim {
//     pub node_name: String,  // Rc<str> ?
//     pub num_position_keys: u32,
//     pub position_keys: Vec<VectorKey>,
//     pub num_rotation_keys: u32,
//     pub rotation_keys: Vec<QuatKey>,
//     pub num_scaling_keys: u32,
//     pub scaling_keys: Vec<VectorKey>,
//     pub pre_state: AnimationBehaviour,
//     pub m_post_state: AnimationBehaviour,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct MeshAnim {
//     pub name: String,
//     pub num_keys: u32,
//     pub keys: Vec<MeshKey>,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct MeshMorphAnim {
//     pub name: String,
//     pub num_keys: u32,
//     pub keys: Vec<MeshMorphKey>,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct Animation {
//     pub name: String,
//     pub duration: f64,
//     pub ticks_per_second: f64,
//     pub num_channels: u32,
//     pub channels: Vec<NodeAnim>,
//     pub num_mesh_channels: u32,
//     pub mesh_channels: Vec<MeshAnim>,
//     pub num_morph_mesh_channels: u32,
//     pub morph_mesh_channels: Vec<MeshMorphAnim>,
// }

// model data
#[derive(Debug, Clone)]
pub struct Model {
    pub name: Rc<str>,
    pub shader: Rc<Shader>,  // todo: remove shader from model since which shader depends the render context
    pub meshes: Rc<Vec<ModelMesh>>,
    pub bone_info_map: Rc<RefCell<HashMap<String, BoneInfo>>>,
    pub bone_count: i32,
    // pub animations: Rc<Vec<Animation>>,
}

impl Model {
    pub fn render(&self) {
        for mesh in self.meshes.iter() {
            mesh.render(&self.shader);
        }
    }

    pub fn render_with_shader(&self, shader: &Rc<Shader>) {
        for mesh in self.meshes.iter() {
            mesh.render(shader);
        }
    }

    pub fn render_with_transform(&self, position: Vec3, angle: f32, scale: Vec3, _delta_time: f32) {
        let mut model_transform = Mat4::from_translation(position);
        model_transform *= Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), angle.to_radians());
        model_transform *= Mat4::from_scale(scale);
        self.shader.set_mat4("model", &model_transform);

        for mesh in self.meshes.iter() {
            mesh.render(&self.shader);
        }
    }
}

#[derive(Debug)]
pub struct ModelBuilder {
    pub name: String,
    pub shader: Rc<Shader>,
    pub meshes: Vec<ModelMesh>,
    pub bone_info_map: Rc<RefCell<HashMap<String, BoneInfo>>>,
    pub bone_count: i32,
    // pub animations: Vec<Animation>,
    pub filepath: String,
    pub directory: PathBuf,
    pub gamma_correction: bool,
    pub flip_v: bool,
    pub textures_cache: Vec<Rc<Texture>>,
    pub diffuse_count: u32,
    pub specular_count: u32,
    pub normal_count: u32,
    pub height_count: u32,
}

impl ModelBuilder {
    pub fn new(name: impl Into<String>, shader: Rc<Shader>, path: impl Into<String>) -> Self {
        let filepath = path.into();
        let directory = PathBuf::from(&filepath).parent().unwrap().to_path_buf();
        ModelBuilder {
            name: name.into(),
            shader,
            textures_cache: vec![],
            meshes: vec![],
            bone_info_map: Rc::new(RefCell::new(HashMap::new())),
            bone_count: 0,
            // animations: vec![],
            filepath,
            directory,
            gamma_correction: false,
            flip_v: false,
            diffuse_count: 0,
            specular_count: 0,
            normal_count: 0,
            height_count: 0,
        }
    }

    pub fn flipv(mut self) -> Self {
        self.flip_v = true;
        self
    }

    pub fn correct_gamma(mut self) -> Self {
        self.gamma_correction = true;
        self
    }

    pub fn build(mut self) -> Result<Model, Error> {
        self.load_model()?;
        let model = Model {
            name: Rc::from(self.name),
            shader: self.shader,
            meshes: Rc::from(self.meshes),
            bone_info_map: self.bone_info_map,
            bone_count: self.bone_count,
            // animations: Rc::from(self.animations),
        };

        Ok(model)
    }

    // loads a model with supported ASSIMP extensions from file and stores the resulting meshes in the meshes vector.
    fn load_model(&mut self) -> Result<(), Error> {
        let path = self.filepath.clone();
        let scene = AssimpScene::from_file(
            &path,
            vec![
                PostProcess::Triangulate,
                PostProcess::GenerateSmoothNormals,
                PostProcess::FlipUVs,
                PostProcess::CalculateTangentSpace,
                // PostProcess::JoinIdenticalVertices,
                // PostProcess::SortByPrimitiveType,
                // PostProcess::EmbedTextures,
            ],
        );

        match scene {
            Ok(scene) => {
                self.walk_animation_nodes(scene.assimp_scene);
                self.process_node(scene.assimp_scene.mRootNode, scene.assimp_scene)
            },
            Err(err) => Err(ModelError(err)),
        }
    }

    // TODO: temp
    fn walk_animation_nodes(&self, scene: &aiScene) {
        let slice = slice_from_raw_parts(scene.mAnimations, scene.mNumAnimations as usize);
        match unsafe { slice.as_ref() } {
            None => {}
            Some(animations) => {
                for i in 0..animations.len() {
                    if unsafe {(*animations[i]).mName.length} > 0 {
                        let name: String = unsafe { (*animations[i]).mName.into() };
                        println!("animation name: {:?}", name);
                    }
                    let animation: Animation = unsafe { (&(*animations[i])).into() };
                    // println!("animation: {:?}", animation);
                    for node in animation.channels {
                        println!("NodeAnim: {:?}", node.name)
                    }
                    println!();
                    for morph in animation.morph_mesh_channels {
                        println!("MeshMorphAnim: {:?}", morph.name)
                    }
                    println!();
                    for mesh in animation.mesh_channels {
                        println!("MeshAnim: {:?}", mesh.name)
                    }
                    println!();
                }
            }
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn process_node(&mut self, node: *mut aiNode, scene: &aiScene) -> Result<(), Error> {
        // process each mesh located at the current node
        // println!("{:?}", unsafe { (*node).mName });

        let slice = slice_from_raw_parts(scene.mMeshes, scene.mNumMeshes as usize);
        let assimp_meshes = unsafe { slice.as_ref() }.unwrap();

        for i in 0..assimp_meshes.len() {
            let mesh = self.process_mesh(assimp_meshes[i], scene);
            self.meshes.push(mesh?);
        }

        // Process children nodes
        let slice =
            unsafe { slice_from_raw_parts((*node).mChildren, (*node).mNumChildren as usize) };

        if let Some(child_nodes) = unsafe { slice.as_ref() } {
            for i in 0..child_nodes.len() {
                self.process_node(child_nodes[i], scene)?;
            }
        }
        Ok(())
    }

    #[allow(clippy::needless_range_loop)]
    fn process_mesh(
        &mut self,
        scene_mesh: *mut aiMesh,
        ai_scene: &aiScene,
    ) -> Result<ModelMesh, Error> {
        let ai_mesh = unsafe { *scene_mesh };

        let mut vertices: Vec<ModelVertex> = vec![];
        let mut indices: Vec<u32> = vec![];
        let mut textures: Vec<TextureSample> = vec![];

        let vertex_vecs = get_vec_from_parts(ai_mesh.mVertices, ai_mesh.mNumVertices);
        let normal_vecs = get_vec_from_parts(ai_mesh.mNormals, ai_mesh.mNumVertices);
        let tangent_vecs = get_vec_from_parts(ai_mesh.mTangents, ai_mesh.mNumVertices);
        let bitangents_vecs = get_vec_from_parts(ai_mesh.mBitangents, ai_mesh.mNumVertices);

        // a vertex can contain up to 8 different texture coordinates. We thus make the assumption that we won't
        // use models where a vertex can have multiple texture coordinates so we always take the first set (0).
        let texture_coords = if !ai_mesh.mTextureCoords.is_empty() {
            get_vec_from_parts(ai_mesh.mTextureCoords[0], vertex_vecs.len() as u32)
        } else {
            vec![]
        };

        for i in 0..vertex_vecs.len() {
            let mut vertex = ModelVertex::new();

            // positions
            vertex.position = vertex_vecs[i]; // Vec3 has Copy trait

            // normals
            if !normal_vecs.is_empty() {
                vertex.normal = normal_vecs[i];
            }

            // texture coordinates
            if !texture_coords.is_empty() {
                vertex.uv = vec2(texture_coords[i].x, texture_coords[i].y);
                vertex.tangent = tangent_vecs[i];
                vertex.bi_tangent = bitangents_vecs[i];
            } else {
                vertex.uv = vec2(0.0, 0.0);
            }
            vertices.push(vertex);
        }
        // now walk through each of the mesh's faces (a face is a mesh its triangle) and retrieve the corresponding vertex indices.
        let assimp_faces = unsafe {
            slice_from_raw_parts(ai_mesh.mFaces, ai_mesh.mNumFaces as usize).as_ref()
        }
            .unwrap();

        for i in 0..assimp_faces.len() {
            let face = assimp_faces[i];
            let assimp_indices =
                unsafe { slice_from_raw_parts(face.mIndices, face.mNumIndices as usize).as_ref() }
                    .unwrap();
            indices.extend(assimp_indices.iter());
        }

        // process materials
        let assimp_materials = unsafe {
            slice_from_raw_parts(ai_scene.mMaterials, ai_scene.mNumMaterials as usize).as_ref()
        }
            .unwrap();
        let material_index = ai_mesh.mMaterialIndex as usize;
        let assimp_material = assimp_materials[material_index];

        // we assume a convention for sampler names in the shaders. Each diffuse texture should be named
        // as 'texture_diffuseN' where N is a sequential number ranging from 1 to MAX_SAMPLER_NUMBER.
        // Same applies to other texture as the following list summarizes:
        // diffuse: texture_diffuseN
        // specular: texture_specularN
        // normal: texture_normalN

        // 1. diffuse maps
        let diffuse_textures = self.load_material_textures(assimp_material, TextureType::Diffuse)?;
        textures.extend(diffuse_textures);

        // 2. specular maps
        let specular_textures = self.load_material_textures(assimp_material, TextureType::Specular)?;
        textures.extend(specular_textures);

        // 3. normal maps
        let normal_textures = self.load_material_textures(assimp_material, TextureType::Normal)?;
        textures.extend(normal_textures);

        // 4. height maps
        let height_maps = self.load_material_textures(assimp_material, TextureType::Height)?;
        textures.extend(height_maps);

        self.extract_bone_weights_for_vertices(&mut vertices, &ai_mesh);

        let mesh = ModelMesh::new(vertices, indices, textures);
        Ok(mesh)
    }

    fn extract_bone_weights_for_vertices(&mut self, vertices: &mut Vec<ModelVertex>, ai_mesh: &aiMesh)  {
        let bones: Vec<russimp::bone::Bone> = russimp::utils::get_vec_from_raw(ai_mesh.mBones, ai_mesh.mNumBones);
        for bone in bones {
            let mut bone_id = -1;
            match self.bone_info_map.borrow().get(&bone.name) {
                None => {
                    let bone_info = BoneInfo {
                        id: self.bone_count,
                        offset: convert_to_mat4(&bone.offset_matrix),
                    };
                    self.bone_info_map.borrow_mut().insert(bone.name.clone(), bone_info);
                    bone_id = self.bone_count;
                    self.bone_count += 1;
                }
                Some(bone_info) => {
                    bone_id = bone_info.id;
                }
            }
            let bone_weights = bone.weights;
            for bone_weight in bone_weights {
                let vertex_id = bone_weight.vertex_id as usize;
                let weight = bone_weight.weight;
                assert!(vertex_id <= vertices.len());
                vertices[vertex_id].set_bone_data(bone_id, weight);
            }
        }
    }

    fn load_material_textures(
        &mut self,
        assimp_material: *mut aiMaterial,
        texture_type: TextureType,
    ) -> Result<Vec<TextureSample>, Error> {
        let mut textures: Vec<TextureSample> = vec![];

        let texture_count =
            unsafe { aiGetMaterialTextureCount(assimp_material, texture_type.into()) };

        // println!("loading texture_count: {}", texture_count);

        for i in 0..texture_count {
            let texture_filename = unsafe { get_material_texture_filename(assimp_material, texture_type, i)? };
            let full_path = self.directory.join(&texture_filename);

            // println!("model texture full_path: {:?}", full_path);

            let cached_texture = self
                .textures_cache
                .iter()
                .find(|t| t.texture_path == full_path.clone().into_os_string());

            if cached_texture.is_some() {
                continue;
            }

            let sample_name = self.get_next_texture_name(texture_type);
            let texture = Rc::new(Texture::new(
                full_path,
                &TextureConfig {
                    flip_v: self.flip_v,
                    gamma_correction: self.gamma_correction,
                    filter: TextureFilter::Linear,
                    wrap: TextureWrap::Clamp,
                    texture_type,
                },
            )?);
            self.textures_cache.push(texture.clone());
            let texture_sample = TextureSample {
                sample_name,
                texture,
            };
            textures.push(texture_sample);
        }
        Ok(textures)
    }

    // todo: revisit setting name here. shader has dependency on name and the order of the textures. hmm.
    fn get_next_texture_name(&mut self, texture_type: TextureType) -> String {
        let num = match texture_type {
            TextureType::Diffuse => {
                self.diffuse_count += 1;
                self.diffuse_count
            }
            TextureType::Specular => {
                self.specular_count += 1;
                self.specular_count
            }
            TextureType::Normal => {
                self.normal_count += 1;
                self.normal_count
            }
            TextureType::Height => {
                self.height_count += 1;
                self.height_count
            }
            _ => todo!(),
        };

        texture_type.to_string().add(&num.to_string())
    }
}

pub fn get_vec_from_parts(raw_data: *mut aiVector3D, size: c_uint) -> Vec<Vec3> {
    let slice = slice_from_raw_parts(raw_data, size as usize);
    if slice.is_null() {
        return vec![];
    }

    let raw_array = unsafe { slice.as_ref() }.unwrap();
    raw_array
        .iter()
        .map(|aiv| vec3(aiv.x, aiv.y, aiv.z))
        .collect()
}
