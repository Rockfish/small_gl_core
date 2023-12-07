use glam::{vec2, vec3};
use small_gl_core::model::Model;
use small_gl_core::model_mesh::{ModelMesh, ModelVertex};
use small_gl_core::shader::Shader;
use small_gl_core::texture::{Texture, TextureConfig, TextureFilter, TextureType, TextureWrap};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Cube {
    mesh: ModelMesh,
}

impl Cube {
    pub fn new(name: &str, shader: Rc<Shader>) -> Model {
        let mesh = Cube::get_meshes();

        let model = Model {
            name: Rc::from(name),
            shader,
            meshes: Rc::new(RefCell::new(vec![mesh])),
            bone_data_map: Rc::new(RefCell::new(Default::default())),
            bone_count: 0,
        };

        model
    }

    fn get_meshes() -> ModelMesh {
        let (vertices, indices) = Cube::data();
        let texture = Rc::new(
            Texture::new(
                "examples/sample_animation/container2.png",
                &TextureConfig {
                    flip_v: false,
                    gamma_correction: false,
                    filter: TextureFilter::Linear,
                    wrap: TextureWrap::Clamp,
                    texture_type: TextureType::Diffuse,
                },
            )
            .unwrap(),
        );

        ModelMesh::new("cube", vertices, indices, vec![texture])
    }

    fn data() -> (Vec<ModelVertex>, Vec<u32>) {
        let mut vertices = [
            ModelVertex {
                position: vec3(1.0, -1.0, 1.0),
                normal: vec3(-0.0, -1.0, 0.0),
                uv: vec2(0.0, 0.0),
                tangent: vec3(1.0, -0.0, -0.0),
                bi_tangent: vec3(-0.0, -0.0, -1.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, -1.0, -1.0),
                normal: vec3(-0.0, -1.0, 0.0),
                uv: vec2(-1.0, 1.0),
                tangent: vec3(1.0, -0.0, -0.0),
                bi_tangent: vec3(-0.0, -0.0, -1.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, -1.0, -1.0),
                normal: vec3(-0.0, -1.0, 0.0),
                uv: vec2(0.0, 1.0),
                tangent: vec3(1.0, -0.0, -0.0),
                bi_tangent: vec3(-0.0, -0.0, -1.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, 1.0, -1.0),
                normal: vec3(0.0, 1.0, -0.0),
                uv: vec2(0.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                bi_tangent: vec3(0.0, -0.0, -1.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, 1.0, 1.0),
                normal: vec3(0.0, 1.0, -0.0),
                uv: vec2(1.0, -1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                bi_tangent: vec3(0.0, -0.0, -1.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, 1.0, -1.0),
                normal: vec3(0.0, 1.0, -0.0),
                uv: vec2(1.0, 0.0),
                tangent: vec3(1.0, -0.0, 0.0),
                bi_tangent: vec3(0.0, -0.0, -1.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, 1.0, -1.0),
                normal: vec3(1.0, -0.0, -0.0),
                uv: vec2(1.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                bi_tangent: vec3(0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, -1.0, 1.0),
                normal: vec3(1.0, -0.0, -0.0),
                uv: vec2(0.0, -1.0),
                tangent: vec3(0.0, 0.0, -1.0),
                bi_tangent: vec3(0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, -1.0, -1.0),
                normal: vec3(1.0, -0.0, -0.0),
                uv: vec2(1.0, -1.0),
                tangent: vec3(0.0, 0.0, -1.0),
                bi_tangent: vec3(0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, 1.0, 1.0),
                normal: vec3(-0.0, -0.0, 1.0),
                uv: vec2(1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                bi_tangent: vec3(-0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, -1.0, 1.0),
                normal: vec3(-0.0, -0.0, 1.0),
                uv: vec2(-0.0, -1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                bi_tangent: vec3(-0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, -1.0, 1.0),
                normal: vec3(-0.0, -0.0, 1.0),
                uv: vec2(1.0, -1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                bi_tangent: vec3(-0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, -1.0, 1.0),
                normal: vec3(-1.0, -0.0, -0.0),
                uv: vec2(0.0, 0.0),
                tangent: vec3(0.0, -0.0, -1.0),
                bi_tangent: vec3(-0.0, 1.0, -0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, 1.0, -1.0),
                normal: vec3(-1.0, -0.0, -0.0),
                uv: vec2(1.0, 1.0),
                tangent: vec3(0.0, -0.0, -1.0),
                bi_tangent: vec3(-0.0, 1.0, -0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, -1.0, -1.0),
                normal: vec3(-1.0, -0.0, -0.0),
                uv: vec2(1.0, 0.0),
                tangent: vec3(0.0, -0.0, -1.0),
                bi_tangent: vec3(-0.0, 1.0, -0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, -1.0, -1.0),
                normal: vec3(0.0, 0.0, -1.0),
                uv: vec2(0.0, 0.0),
                tangent: vec3(1.0, -0.0, 0.0),
                bi_tangent: vec3(-0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, 1.0, -1.0),
                normal: vec3(0.0, 0.0, -1.0),
                uv: vec2(-1.0, 1.0),
                tangent: vec3(1.0, -0.0, 0.0),
                bi_tangent: vec3(-0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, 1.0, -1.0),
                normal: vec3(0.0, 0.0, -1.0),
                uv: vec2(0.0, 1.0),
                tangent: vec3(1.0, -0.0, 0.0),
                bi_tangent: vec3(-0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, -1.0, 1.0),
                normal: vec3(0.0, -1.0, 0.0),
                uv: vec2(0.0, 0.0),
                tangent: vec3(1.0, -0.0, -0.0),
                bi_tangent: vec3(-0.0, -0.0, -1.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, -1.0, 1.0),
                normal: vec3(0.0, -1.0, 0.0),
                uv: vec2(-1.0, 0.0),
                tangent: vec3(1.0, 0.0, -0.0),
                bi_tangent: vec3(-0.0, -0.0, -1.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, -1.0, -1.0),
                normal: vec3(0.0, -1.0, 0.0),
                uv: vec2(-1.0, 1.0),
                tangent: vec3(1.0, -0.0, -0.0),
                bi_tangent: vec3(-0.0, -0.0, -1.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, 1.0, -1.0),
                normal: vec3(0.0, 1.0, 0.0),
                uv: vec2(0.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                bi_tangent: vec3(0.0, 0.0, -1.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, 1.0, 1.0),
                normal: vec3(0.0, 1.0, 0.0),
                uv: vec2(-0.0, -1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                bi_tangent: vec3(0.0, 0.0, -1.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, 1.0, 1.0),
                normal: vec3(0.0, 1.0, 0.0),
                uv: vec2(1.0, -1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                bi_tangent: vec3(0.0, 0.0, -1.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, 1.0, -1.0),
                normal: vec3(1.0, 0.0, 1e-6),
                uv: vec2(1.0, 0.0),
                tangent: vec3(1e-6, 0.0, -1.0),
                bi_tangent: vec3(-0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, 1.0, 1.0),
                normal: vec3(1.0, 0.0, 1e-6),
                uv: vec2(-0.0, 0.0),
                tangent: vec3(1e-6, 0.0, -1.0),
                bi_tangent: vec3(-0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, -1.0, 1.0),
                normal: vec3(1.0, 0.0, 1e-6),
                uv: vec2(0.0, -1.0),
                tangent: vec3(1e-6, 0.0, -1.0),
                bi_tangent: vec3(-0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, 1.0, 1.0),
                normal: vec3(-0.0, 0.0, 1.0),
                uv: vec2(1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                bi_tangent: vec3(0.0, 1.0, -0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, 1.0, 1.0),
                normal: vec3(-0.0, 0.0, 1.0),
                uv: vec2(-0.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                bi_tangent: vec3(0.0, 1.0, -0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, -1.0, 1.0),
                normal: vec3(-0.0, 0.0, 1.0),
                uv: vec2(-0.0, -1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                bi_tangent: vec3(0.0, 1.0, -0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, -1.0, 1.0),
                normal: vec3(-1.0, -0.0, -0.0),
                uv: vec2(0.0, 0.0),
                tangent: vec3(0.0, -0.0, -1.0),
                bi_tangent: vec3(-0.0, 1.0, -0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, 1.0, 1.0),
                normal: vec3(-1.0, -0.0, -0.0),
                uv: vec2(0.0, 1.0),
                tangent: vec3(0.0, -0.0, -1.0),
                bi_tangent: vec3(-0.0, 1.0, -0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, 1.0, -1.0),
                normal: vec3(-1.0, -0.0, -0.0),
                uv: vec2(1.0, 1.0),
                tangent: vec3(0.0, -0.0, -1.0),
                bi_tangent: vec3(-0.0, 1.0, -0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(1.0, -1.0, -1.0),
                normal: vec3(0.0, 0.0, -1.0),
                uv: vec2(0.0, 0.0),
                tangent: vec3(1.0, -0.0, 0.0),
                bi_tangent: vec3(-0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, -1.0, -1.0),
                normal: vec3(0.0, 0.0, -1.0),
                uv: vec2(-1.0, 0.0),
                tangent: vec3(1.0, -0.0, 0.0),
                bi_tangent: vec3(-0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: vec3(-1.0, 1.0, -1.0),
                normal: vec3(0.0, 0.0, -1.0),
                uv: vec2(-1.0, 1.0),
                tangent: vec3(1.0, -0.0, 0.0),
                bi_tangent: vec3(-0.0, 1.0, 0.0),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            },
        ]
        .to_vec();

        for vert in vertices.iter_mut() {
            vert.bone_ids = [0, -1, -1, -1];
            vert.bone_weights = [1.0, 0.0, 0.0, 0.0];
        }

        let indices: Vec<u32> =
            [
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
                33, 34, 35,
            ]
            .to_vec();

        (vertices, indices)
    }
}
