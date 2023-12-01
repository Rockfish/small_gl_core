use std::collections::HashMap;
use std::rc::Rc;
use russimp::animation::Animation;
use russimp::bone::Bone;
use russimp::Matrix4x4;
use russimp::mesh::Mesh;
use russimp::node::Node;
use russimp::scene::Scene;
use russimp::sys::aiMatrix4x4;
use small_gl_core::assimp_scene::AssimpScene;

fn main() {
    let model_path = "examples/sample_animation/animated_cube/AnimatedCube.gltf";
    // let model_path = "examples/sample_animation/source/cube_capoeira_martelo_cruzando.FBX.fbx";
    // let model_path = "/Users/john/Dev_Rust/Repos/ogldev/Content/box.obj";
    let model_path = "/Users/john/Dev_Rust/Repos/OpenGL-Animation/Resources/res/model.dae";
    // let model_path = "examples/sample_animation/colorful_cube/scene.gltf";
    let scene = AssimpScene::load_assimp_scene(model_path).unwrap();

    let mut parse = SceneParser::new();
    parse.parse_scene(&scene);
}

const MAX_NUM_BONES_PER_VERTEX: i32 = 4;

struct VertexBoneData {
    bone_ids: Vec<i32>,
    weights: Vec<f32>,
    index: i32,
}

impl VertexBoneData {
    pub fn new() -> Self {
        VertexBoneData {
            bone_ids: vec![-1; 100],
            weights: vec![0.0; 100],
            index: 0,
        }
    }

    pub fn add_bone_data(&mut self, bone_id: i32, weight: f32) {
        for i in 0..self.index {
            if self.bone_ids[i as usize] == bone_id {
                println!("bone {} already found at index {} old weight {} new weight {}\n", bone_id, i, self.weights[i as usize], weight);
                return;
            }
        }

        println!("bone {} weight {} at index {}\n", bone_id, weight, self.index);

        if self.index >= MAX_NUM_BONES_PER_VERTEX {
            println!("Warning: exceeding the maximum number of bones per vertex (current index {})\n", self.index);
        }

        if self.index < 100 {
            self.bone_ids[self.index as usize] = bone_id;
            self.weights[self.index as usize] = weight;
        } else {
            println!("bone_id: {} exceeds allocated vec for bones_ids", bone_id);
        }

        self.index += 1;
    }

    pub fn get_weight_sum(&self) -> f32 {
        let mut sum: f32 = 0.0;
        for i in 0..self.index {
            sum += self.weights[i as usize];
        }
        sum
    }
}

struct SceneParser {
    vertex_to_bones: Vec<VertexBoneData>,
    mesh_base_vertex: Vec<i32>,
    bone_name_to_index_map: HashMap<String, u32>,
    space_count: i32,
}

impl SceneParser {
    pub fn new() -> Self {
        SceneParser {
            vertex_to_bones: vec![],
            mesh_base_vertex: vec![],
            bone_name_to_index_map: Default::default(),
            space_count: 0,
        }
    }

    fn parse_scene(&mut self, assimp_scene: &AssimpScene) {
        let ai_scene = unsafe { assimp_scene.assimp_scene.as_ref() }.unwrap();
        let scene = Scene::new(ai_scene).unwrap();

        self.parse_meshes(&scene);
        self.validate_bones();
        self.parse_hierarchy(&scene);
        self.parse_animations(&scene);
    }

    fn parse_meshes(&mut self, scene: &Scene) {
        println!("---------------------------------------------");
        println!("Parsing {} meshes\n\n", &scene.meshes.len());

        let mut total_vertices: i32 = 0;
        let mut total_indices: i32 = 0;
        let mut total_bones: i32 = 0;

        for (i, mesh) in scene.meshes.iter().enumerate() {
            let num_vertices = mesh.vertices.len();
            let num_indices = mesh.faces.len() * 3;
            let num_bones = mesh.bones.len();

            println!("  Mesh {} '{}': vertices {} indices {} bones {}\n", i, mesh.name, num_vertices, num_indices, num_bones);

            total_vertices += num_vertices as i32;
            total_indices  += num_indices as i32;
            total_bones += num_bones as i32;
            // self.vertex_to_bones.push(total_vertices)

            self.parse_single_mesh(i, mesh);
        }
    }

    fn parse_single_mesh(&mut self, mesh_index: usize, mesh: &Mesh) {
        println!("Vertex positions\n");

        for (i, vert) in mesh.vertices.iter().enumerate() {
            println!("{} :  {} {} {}", i, vert.x, vert.y, vert.z);
            if i > 10 {
                println!("... skipping {} vertices: ", mesh.vertices.len() - i);
                break;
            }
        }

        println!("\nIndices\n");
        for (i, face) in mesh.faces.iter().enumerate() {
            println!("{} : {:?}", i, face);
            if i > 10 {
                println!("... skipping {} indices: ", mesh.faces.len() - i);
                break;
            }
        }

        println!("\nBones number: {}", mesh.bones.len());
        self.parse_mesh_bones(mesh_index, mesh);

        println!();
    }

    fn parse_mesh_bones(&mut self, mesh_index: usize, mesh: &Mesh) {
        for bone in &mesh.bones {
            self.parse_bone(mesh_index, bone);
        }
    }

    fn parse_bone(&mut self, mesh_index: usize, bone: &Bone) {
        println!("      Bone '{}': num vertices affected by this bone: {}", bone.name, bone.weights.len());

        let bone_id = self.get_bone_id(bone);

        self.print_assimp_matrix(&bone.offset_matrix);

        for (i, weight) in bone.weights.iter().enumerate() {
            print!("     {} : vertex id {} ", i, weight.vertex_id);

            let global_vertex_id = self.mesh_base_vertex[i] + weight.vertex_id as i32;

            // assert(global_vertex_id < vertex_to_bones.size());
            // vertex_to_bones[global_vertex_id].AddBoneData(bone_id, vw.mWeight);
        }

        println!();
    }

    fn get_bone_id(&mut self, bone: &Bone) -> u32 {
        let mut bone_id = 0;
        match self.bone_name_to_index_map.get(bone.name.as_str()) {
            None => {
                bone_id = self.bone_name_to_index_map.len() as u32;
                self.bone_name_to_index_map.insert(bone.name.clone(), bone_id);
            }
            Some(id) => bone_id = *id,
        }
        bone_id
    }

    fn parse_hierarchy(&mut self, scene: &Scene) {
        println!("\n*******************************************************");
        println!("Parsing the node hierarchy");
        self.parse_node(&scene.root.as_ref().unwrap());
    }

    fn parse_node(&mut self, node: &Rc<Node>) {
        self.print_space();
        println!("Node name: '{}' num children {} num meshes {}", node.name, node.children.borrow().len(), node.meshes.len());
        self.print_space();
        println!("Node transformation:");
        self.print_assimp_matrix(&node.transformation);

        self.space_count += 4;

        for i in 0..node.children.borrow().len() {
            println!();
            self.print_space();
            println!("--- {} ---\n", i);
            self.parse_node(node.children.borrow().get(i).unwrap());
        }

        self.space_count -= 4;
    }

    fn parse_animations(&mut self, scene: &Scene) {
        println!("\n*******************************************************");
        println!("Parsing animations\n");

        for (i, animation) in scene.animations.iter().enumerate() {
            self.parse_single_animation(i, animation);
        }
        println!();
    }

    fn parse_single_animation(&mut self, animation_id: usize, animation: &Animation) {

        println!("animation: {}   name: {}", animation_id, animation.name);
        println!("ticks_per_second: {}  duration: {}", animation.ticks_per_second, animation.duration);
        println!("NodeAdmin channel length: {}\n", animation.channels.len());

        for (i, channel) in animation.channels.iter().enumerate() {
            println!("channel id: {}  name: {}  position keys: {}  rotation keys: {}, scaling keys: {}", i, channel.name, channel.position_keys.len(), channel.rotation_keys.len(), channel.scaling_keys.len());
        }
        println!();
    }

    fn validate_bones(&self) {
        println!("Validating bones");
        for i in 0..self.vertex_to_bones.len() {
            println!("{}: {}", i, self.vertex_to_bones[i].get_weight_sum());
        }
    }

    fn print_space(&self) {
        for i in 0..self.space_count {
            print!(" ");
        }
    }

    fn print_assimp_matrix(&self, m: &Matrix4x4) {
        self.print_space(); print!("{} {} {} {}\n", m.a1, m.a2, m.a3, m.a4);
        self.print_space(); print!("{} {} {} {}\n", m.b1, m.b2, m.b3, m.b4);
        self.print_space(); print!("{} {} {} {}\n", m.c1, m.c2, m.c3, m.c4);
        self.print_space(); print!("{} {} {} {}\n", m.d1, m.d2, m.d3, m.d4);
    }
}