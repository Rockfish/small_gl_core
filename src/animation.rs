use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use glam::{Mat4, vec4};
use russimp::animation::Animation;
use russimp::Matrix4x4;
use russimp::node::Node;
use russimp::scene::Scene;
use crate::assimp_scene::AssimpScene;
use crate::bone::{Bone, BoneInfo};
use crate::model::Model;

pub struct NodeData {
    pub name: String,
    pub transformation: Mat4,
    pub children: Vec<NodeData>,
}

impl NodeData {
    pub fn new() -> Self {
        NodeData {
            name: String::new(),
            transformation: Default::default(),
            children: vec![],
        }
    }
}

pub struct ModelAnimation {
    pub duration: f32,
    pub ticks_per_second: f32,
    pub root_node: NodeData,
    pub bones: Vec<Bone>,
    pub bone_info_map: Rc<RefCell<HashMap<String, BoneInfo>>>
}

impl ModelAnimation {
    pub fn new(assimp_scene: AssimpScene, model: &mut Model) -> Self {
        let scene = Scene::new(assimp_scene.assimp_scene).unwrap();
        let duration = scene.animations[0].duration as f32;
        let ticks_per_second = scene.animations[0].ticks_per_second as f32;
        let root = scene.root.as_ref().unwrap().clone();
        let mut root_node = NodeData::new();
        ModelAnimation::read_hierarchy_data(&mut root_node, &root);
        let mut model_animation = ModelAnimation {
            duration,
            ticks_per_second,
            root_node,
            bones: vec![],
            bone_info_map: model.bone_info_map.clone(),
        };
        model_animation.read_channel_bones(&scene.animations[0], model);
        model_animation
    }

    fn read_hierarchy_data(dest: &mut NodeData, source: &Rc<Node>) {
        dest.name = source.name.clone();
        dest.transformation = convert_to_mat4(&source.transformation);
        for child in source.children.borrow().iter() {
            let mut node = NodeData::new();
            ModelAnimation::read_hierarchy_data(&mut node, child);
            dest.children.push(node);
        }
    }

    // todo: step through this to see if it makes sense
    fn read_channel_bones(&mut self, animation: &Animation, model: &mut Model) {
        for channel in &animation.channels {

            // todo: revisit, added an insert that seems to be missing in the original code.
            if model.bone_info_map.borrow().get(&channel.name).is_none() {
                // todo: not sure since the new bone info doesn't have a real offset
                let bone_info = BoneInfo::new(model.bone_count, Mat4::IDENTITY);
                model.bone_info_map.borrow_mut().insert(channel.name.clone(), bone_info);
                model.bone_count += 1;
            }

            let id = model.bone_info_map.borrow().get(&channel.name).unwrap().id;
            let bone = Bone::new( &channel.name.clone(), id, &channel);
            self.bones.push(bone);
        }
        // self.bone_info_map = model.bone_info_map - don't need this because they are already sharing the same map
    }
}

pub fn convert_to_mat4(m: &Matrix4x4) -> Mat4 {
     Mat4::from_cols(
        vec4(m.a1, m.a2, m.a3, m.a4),
        vec4(m.b1, m.b2, m.b3, m.b4),
        vec4(m.c1, m.c2, m.c3, m.c4),
        vec4(m.d1, m.d2, m.d3, m.d4),
    )
}