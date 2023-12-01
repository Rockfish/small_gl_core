use crate::assimp_scene::AssimpScene;
use crate::bone_data::{BoneAnimation, BoneData};
use crate::model::{BoneName, Model};
use glam::Mat4;
use russimp::animation::Animation;
use russimp::node::Node;
use russimp::scene::Scene;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::assimp_utils::convert_to_mat4;

/// NodeData is local version of aiNode
#[derive(Debug)]
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
    pub bone_animations: RefCell<Vec<BoneAnimation>>,
    pub bone_data_map: Rc<RefCell<HashMap<BoneName, BoneData>>>,
    pub global_inverse_transform: Mat4,
}

impl ModelAnimation {
    pub fn new(assimp_scene: &AssimpScene, model: &mut Model) -> Self {
        let ai_scene = unsafe { assimp_scene.assimp_scene.as_ref() }.unwrap();
        let scene = Scene::new(ai_scene).unwrap();

        let duration = scene.animations[0].duration as f32;
        let ticks_per_second = scene.animations[0].ticks_per_second as f32;

        let root = scene.root.as_ref().unwrap().clone();
        let global_inverse_transform = convert_to_mat4(&root.transformation).inverse();

        let root_node = ModelAnimation::read_hierarchy_data(&root);

        // println!("root_node: {:#?}", &root_node);
        // println!("bone_data_map: {:#?}", &model.bone_data_map.borrow());

        let mut model_animation = ModelAnimation {
            duration,
            ticks_per_second,
            root_node,
            bone_animations: vec![].into(),
            bone_data_map: model.bone_data_map.clone(),
            global_inverse_transform
        };

        model_animation.read_channel_bone_animations(&scene.animations[0], model);
        model_animation
    }

    /// Converts scene.aiNode tree to local NodeData tree. Converting all the transforms to column major form.
    fn read_hierarchy_data(source: &Rc<Node>) -> NodeData {

        let mut node_data = NodeData {
            name: source.name.clone(),
            transformation: convert_to_mat4(&source.transformation),
            children: vec![]
        };

        // println!("NodeData name: {}\n transform: {:?}\n", &node_data.name, &node_data.transformation);

        for child in source.children.borrow().iter() {
            let node = ModelAnimation::read_hierarchy_data(child);
            node_data.children.push(node);
        }
        node_data
    }

    fn read_channel_bone_animations(&mut self, animation: &Animation, model: &mut Model) {
        let mut bone_data_map = model.bone_data_map.borrow_mut();

        for channel in &animation.channels {

            if bone_data_map.get(&channel.name).is_none() {

                println!("Bone_Data not found for channel name: {}", &channel.name);

                let bone_info = BoneData::new(&channel.name.clone(), model.bone_count, Mat4::IDENTITY);

                bone_data_map.insert(channel.name.clone(), bone_info);
                model.bone_count += 1;
            }

            let bone_data = &bone_data_map[&channel.name];

            let bone_index = bone_data.bone_index;
            let bone = BoneAnimation::new(&channel.name.clone(), bone_index, &channel);

            self.bone_animations.borrow_mut().push(bone);
        }
    }
}

