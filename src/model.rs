use crate::error::Error;
use crate::error::Error::{MeshError, SceneError};
use crate::model_mesh::{ModelMesh, ModelVertex};
use crate::node_animation::BoneData;
use crate::shader::Shader;
use crate::texture::{Texture, TextureConfig, TextureFilter, TextureType, TextureWrap};
use crate::transform::Transform;
use crate::utils::HashMap;
use glam::*;
use russimp::node::Node;
use russimp::scene::{PostProcess, Scene};
use russimp::sys::*;
use std::cell::RefCell;
use std::os::raw::c_uint;
use std::path::PathBuf;
use std::ptr::*;
use std::rc::Rc;

pub type BoneName = String;

// model data
#[derive(Debug, Clone)]
pub struct Model {
    pub name: Rc<str>,
    pub shader: Rc<Shader>,
    // todo: remove shader from model since which shader depends the render context
    pub meshes: Rc<RefCell<Vec<ModelMesh>>>,
    pub bone_data_map: Rc<RefCell<HashMap<BoneName, BoneData>>>,
    pub bone_count: i32,
    // pub animations: Rc<Vec<Animation>>,
}

impl Default for Model {
    fn default() -> Self {
        Model {
            name: Rc::from(""),
            shader: Rc::new(Default::default()),
            meshes: Rc::new(RefCell::new(vec![])),
            bone_data_map: Rc::new(RefCell::new(Default::default())),
            bone_count: 0,
        }
    }
}

impl Model {
    pub fn render(&self) {
        for mesh in self.meshes.borrow_mut().iter() {
            mesh.render(&self.shader);
        }
    }

    pub fn render_with_shader(&self, shader: &Rc<Shader>) {
        for mesh in self.meshes.borrow_mut().iter() {
            mesh.render(shader);
        }
    }

    pub fn render_with_transform(&self, position: Vec3, angle: f32, scale: Vec3, _delta_time: f32) {
        let mut model_transform = Mat4::from_translation(position);
        model_transform *= Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), angle.to_radians());
        model_transform *= Mat4::from_scale(scale);
        self.shader.set_mat4("model", &model_transform);

        for mesh in self.meshes.borrow_mut().iter() {
            mesh.render(&self.shader);
        }
    }
}

#[derive(Debug)]
struct AddedTextures {
    mesh_name: String,
    texture_type: TextureType,
    texture_filename: String,
}

#[derive(Debug)]
struct AddedBone {
    mesh_name: String,
    bone_name: String,
    bone_weight: f32,
}

#[derive(Debug)]
pub struct ModelBuilder {
    pub name: String,
    pub shader: Rc<Shader>,
    pub meshes: Vec<ModelMesh>,
    pub bone_data_map: Rc<RefCell<HashMap<String, BoneData>>>,
    pub bone_count: i32,
    // pub animations: Vec<Animation>,
    pub filepath: String,
    pub directory: PathBuf,
    pub gamma_correction: bool,
    pub flip_v: bool,
    pub textures_cache: RefCell<Vec<Rc<Texture>>>,
    pub added_textures: Vec<AddedTextures>,
    pub added_bones: Vec<AddedBone>,
    pub mesh_count: i32,
}

impl ModelBuilder {
    pub fn new(name: impl Into<String>, shader: Rc<Shader>, path: impl Into<String>) -> Self {
        let filepath = path.into();
        let directory = PathBuf::from(&filepath).parent().unwrap().to_path_buf();
        ModelBuilder {
            name: name.into(),
            shader,
            textures_cache: RefCell::new(vec![]),
            meshes: vec![],
            bone_data_map: Rc::new(RefCell::new(HashMap::new())),
            bone_count: 0,
            // animations: vec![],
            filepath,
            directory,
            gamma_correction: false,
            flip_v: false,
            added_textures: vec![],
            added_bones: vec![],
            mesh_count: 0,
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

    pub fn add_texture(mut self, mesh_name: impl Into<String>, texture_type: TextureType, texture_filename: impl Into<String>) -> Self {
        let added_texture = AddedTextures {
            mesh_name: mesh_name.into(),
            texture_type,
            texture_filename: texture_filename.into(),
        };
        self.added_textures.push(added_texture);
        self
    }

    pub fn add_bone(mut self, mesh_name: impl Into<String>, bone_name: impl Into<String>, bone_weight: f32) -> Self {
        let added_bone = AddedBone {
            mesh_name: mesh_name.into(),
            bone_name: bone_name.into(),
            bone_weight,
        };
        self.added_bones.push(added_bone);
        self
    }

    pub fn build(mut self) -> Result<Model, Error> {
        let scene = ModelBuilder::load_russimp_scene(self.filepath.as_str())?;

        self.load_model(&scene)?;

        self.add_textures()?;

        let model = Model {
            name: Rc::from(self.name),
            shader: self.shader,
            meshes: Rc::from(RefCell::new(self.meshes)),
            bone_data_map: self.bone_data_map,
            bone_count: self.bone_count,
            // animations: Rc::from(self.animations),
        };

        Ok(model)
    }

    pub fn build_with_scene(mut self, scene: &Scene) -> Result<Model, Error> {
        self.load_model(scene)?;

        self.add_textures()?;

        let model = Model {
            name: Rc::from(self.name),
            shader: self.shader,
            meshes: Rc::from(RefCell::new(self.meshes)),
            bone_data_map: self.bone_data_map,
            bone_count: self.bone_count,
            // animations: Rc::from(self.animations),
        };

        Ok(model)
    }

    pub fn load_russimp_scene(file_path: &str) -> Result<Scene, Error> {
        let scene =
            Scene::from_file(
                file_path,
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
            )?;
        Ok(scene)
    }

    // loads a model with supported ASSIMP extensions from file and stores the resulting meshes in the meshes vector.
    fn load_model(&mut self, scene: &Scene) -> Result<(), Error> {
        match &scene.root {
            None => Err(SceneError("Error getting scene root node".to_string())),
            Some(root_node) => self.process_node(root_node, scene),
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn process_node(&mut self, node: &Rc<Node>, scene: &Scene) -> Result<(), Error> {
        for mesh_id in &node.meshes {
            let scene_mesh = &scene.meshes[*mesh_id as usize];
            let mesh = self.process_mesh(scene_mesh, scene);
            self.meshes.push(mesh?);
        }

        for child_node in node.children.borrow().iter() {
            self.process_node(child_node, scene)?;
        }

        Ok(())
    }

    #[allow(clippy::needless_range_loop)]
    fn process_mesh(&mut self, r_mesh: &russimp::mesh::Mesh, scene: &Scene) -> Result<ModelMesh, Error> {
        let mut vertices: Vec<ModelVertex> = vec![];
        let mut indices: Vec<u32> = vec![];
        let mut textures: Vec<Rc<Texture>> = vec![];

        // a vertex can contain up to 8 different texture coordinates. We thus make the assumption that we won't
        // use models where a vertex can have multiple texture coordinates so we always take the first set (0).

        // let texture_coords = if !ai_mesh.mTextureCoords.is_empty() {
        //     get_vec_from_parts(ai_mesh.mTextureCoords[0], vertex_vec.len() as u32)
        // } else {
        //     vec![]
        // };

        for i in 0..r_mesh.vertices.len() {
            let mut vertex = ModelVertex::new();

            // positions
            vertex.position = r_mesh.vertices[i]; // Vec3 has Copy trait

            // normals
            if !r_mesh.normals.is_empty() {
                vertex.normal = r_mesh.normals[i];
            }

            // texture coordinates
            if !r_mesh.texture_coords.is_empty() {
                let tex_coords = r_mesh.texture_coords[0].as_ref().unwrap();
                vertex.uv = vec2(tex_coords[i].x, tex_coords[i].y);
                vertex.tangent = r_mesh.tangents[i];
                vertex.bi_tangent = r_mesh.bitangents[i];
            }
            vertices.push(vertex);
        }

        for face in &r_mesh.faces {
            indices.extend(&face.0)
        }

        let material = &scene.materials[r_mesh.material_index as usize];

        // we assume a convention for sampler names in the shaders. Each diffuse texture should be named
        // as 'texture_diffuseN' where N is a sequential number ranging from 1 to MAX_SAMPLER_NUMBER.
        // Same applies to other texture as the following list summarizes:
        // diffuse: texture_diffuseN
        // specular: texture_specularN
        // normal: texture_normalN

        // // 1. diffuse maps
        // let diffuse_textures = self.load_material_textures(material, TextureType::Diffuse)?;
        // textures.extend(diffuse_textures);
        //
        // // 2. specular maps
        // let specular_textures = self.load_material_textures(material, TextureType::Specular)?;
        // textures.extend(specular_textures);
        //
        // // 3. normal maps
        // let normal_textures = self.load_material_textures(material, TextureType::Normals)?;
        // textures.extend(normal_textures);
        //
        // // 4. height maps
        // let height_maps = self.load_material_textures(material, TextureType::Height)?;
        // textures.extend(height_maps);

        for (r_texture_type, r_texture) in material.textures.iter() {
            let texture_type = TextureType::convert_from(r_texture_type);
            let texture = self.load_texture(&texture_type, r_texture.borrow().filename.as_str())?;
            textures.push(texture);
        }

        println!("mesh name: {}", &r_mesh.name);

        self.extract_bone_weights_for_vertices(&mut vertices, r_mesh);

        self.add_bones(&r_mesh.name, &mut vertices)?;

        let mesh = ModelMesh::new(self.mesh_count, &r_mesh.name, vertices, indices, textures);
        self.mesh_count += 1;
        Ok(mesh)
    }

    fn extract_bone_weights_for_vertices(&mut self, vertices: &mut Vec<ModelVertex>, r_mesh: &russimp::mesh::Mesh) {
        let mut bone_data_map = self.bone_data_map.borrow_mut();

        for bone in &r_mesh.bones {
            let mut bone_id = -1;

            match bone_data_map.get(&bone.name) {
                None => {
                    // let other_offset = convert_matrix(&bone.offset_matrix);
                    let bone_info = BoneData {
                        name: bone.name.clone(),
                        bone_index: self.bone_count,
                        // offset: convert_to_mat4(&bone.offset_matrix),
                        offset: bone.offset_matrix.clone(),
                        offset_transform: Transform::from_matrix(bone.offset_matrix.clone()),
                    };
                    bone_data_map.insert(bone.name.clone(), bone_info);
                    bone_id = self.bone_count;
                    self.bone_count += 1;
                }

                Some(bone_info) => {
                    bone_id = bone_info.bone_index;
                }
            }

            // let mut last_bone_id = -1;
            for bone_weight in &bone.weights {
                let vertex_id = bone_weight.vertex_id as usize;
                let weight = bone_weight.weight;

                assert!(vertex_id <= vertices.len());

                vertices[vertex_id].set_bone_data(bone_id, weight);

                // debug
                // if bone_id != last_bone_id {
                //     println!("vertex_id: {}  bone_id: {}  weight: {}", vertex_id, bone_id, weight);
                //     last_bone_id = bone_id;
                // }
            }
        }
    }

    // fn load_material_textures(
    //     &mut self,
    //     r_material: russimp::material::Material,
    //     texture_type: TextureType,
    // ) -> Result<Vec<Rc<Texture>>, Error> {
    //     let mut textures: Vec<Rc<Texture>> = vec![];
    //
    //     for (r_texture_type, r_texture) in r_material.textures.iter() {
    //         let texture_type = TextureType::convert_from(r_texture_type);
    //         let texture = self.load_texture(texture_type, r_texture.borrow().filename.as_str())?;
    //         textures.push(texture);
    //     }
    //
    //
    //     if let Some(textures) = option_texture_vec {
    //         for texture in textures.bo
    //     }
    //
    //
    //     let texture_count = unsafe { aiGetMaterialTextureCount(assimp_material, texture_type.into()) };
    //
    //     for i in 0..texture_count {
    //         let texture_filename =
    //             unsafe { get_material_texture_filename(assimp_material, texture_type, i)? };
    //
    //         let texture = self.load_texture(texture_type, texture_filename.as_str())?;
    //         textures.push(texture);
    //     }
    //     Ok(textures)
    // }

    fn add_textures(&mut self) -> Result<(), Error> {
        for added_texture in &self.added_textures {
            let texture = self.load_texture(&added_texture.texture_type, added_texture.texture_filename.as_str())?;
            let mut mesh = self.meshes.iter_mut().find(|mesh| mesh.name == added_texture.mesh_name);
            if let Some(mut model_mesh) = mesh {
                let path = self.directory.join(&added_texture.texture_filename).into_os_string();
                if model_mesh.textures.iter().find(|t| t.texture_path == path).is_none() {
                    model_mesh.textures.push(texture);
                }
            } else {
                return Err(MeshError(format!("add_texture mesh: {} not found", &added_texture.mesh_name)));
            }
        }
        Ok(())
    }

    fn add_bones(&mut self, mesh_name: &String, vertices: &mut Vec<ModelVertex>) -> Result<(), Error> {
        for added_bone in &self.added_bones {
            if added_bone.mesh_name == *mesh_name {
                let bone_map = self.bone_data_map.borrow();
                let option_bone_data = bone_map.get(&added_bone.bone_name);
                if let Some(bone_data) = option_bone_data {
                    for vertex in vertices.iter_mut() {
                        vertex.set_bone_data(bone_data.bone_index, added_bone.bone_weight);
                    }
                } else {
                    return Err(MeshError(format!("add_bones bone: {} not found", &added_bone.bone_name)));
                }
            }
        }
        Ok(())
    }

    /// load or retrieve copy of texture
    fn load_texture(&self, texture_type: &TextureType, texture_filename: &str) -> Result<Rc<Texture>, Error> {
        let full_path = self.directory.join(&texture_filename);

        let mut texture_cache = self.textures_cache.borrow_mut();

        let cached_texture = texture_cache.iter().find(|t| t.texture_path == full_path.clone().into_os_string());

        match cached_texture {
            None => {
                let texture = Rc::new(Texture::new(
                    full_path,
                    &TextureConfig {
                        flip_v: self.flip_v,
                        gamma_correction: self.gamma_correction,
                        filter: TextureFilter::Linear,
                        wrap: TextureWrap::Clamp,
                        texture_type: texture_type.clone(),
                    },
                )?);

                println!("loaded texture: {:?}", &texture);

                texture_cache.push(texture.clone());
                Ok(texture)
            }
            Some(texture) => Ok(texture.clone()),
        }
    }
}

pub fn get_vec_from_parts(raw_data: *mut aiVector3D, size: c_uint) -> Vec<Vec3> {
    let slice = slice_from_raw_parts(raw_data, size as usize);
    if slice.is_null() {
        return vec![];
    }

    let raw_array = unsafe { slice.as_ref() }.unwrap();
    raw_array.iter().map(|aiv| vec3(aiv.x, aiv.y, aiv.z)).collect()
}
