use crate::node_animation::{NodeAnimation, BoneData};
use crate::model::{BoneName, Model};
use glam::Mat4;
use russimp::animation::Animation;
use russimp::node::Node;
use russimp::scene::Scene;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// NodeData is local version of aiNode
#[derive(Debug)]
pub struct NodeData {
    pub name: String,
    pub transformation: Mat4,
    pub children: Vec<NodeData>,
    pub meshes: Vec<u32>,
}

impl NodeData {
    pub fn new() -> Self {
        NodeData {
            name: String::new(),
            transformation: Mat4::IDENTITY,
            children: vec![],
            meshes: vec![],
        }
    }
}

impl Default for NodeData {
    fn default() -> Self {
        NodeData {
            name: String::new(),
            transformation: Mat4::IDENTITY,
            children: vec![],
            meshes: vec![],
        }
    }
}

pub struct ModelAnimation {
    pub model: Rc<RefCell<Model>>,
    pub duration: f32,
    pub ticks_per_second: f32,
    pub root_node: NodeData,
    pub node_animations: RefCell<Vec<NodeAnimation>>,
    pub bone_data_map: Rc<RefCell<HashMap<BoneName, BoneData>>>,
    pub global_inverse_transform: Mat4,
}

impl Default for ModelAnimation {
    fn default() -> Self {
        ModelAnimation {
            model: Rc::new(RefCell::new(Default::default())),
            duration: 0.0,
            ticks_per_second: 0.0,
            root_node: Default::default(),
            node_animations: RefCell::new(vec![]),
            bone_data_map: Rc::new(RefCell::new(Default::default())),
            global_inverse_transform: Default::default(),
        }
    }
}

impl ModelAnimation {
    pub fn new(scene: &Scene, model: Rc<RefCell<Model>>) -> Self {

        if scene.animations.is_empty() {
            return ModelAnimation::default();
        }

        let duration = scene.animations[0].duration as f32;
        let ticks_per_second = scene.animations[0].ticks_per_second as f32;

        println!("animation - duration: {}   ticks_per_second: {}", &duration, &ticks_per_second);

        let root = scene.root.as_ref().unwrap().clone();
        // let global_inverse_transform = convert_to_mat4(&root.transformation).inverse();
        let global_inverse_transform = root.transformation.inverse();

        let root_node = ModelAnimation::read_hierarchy_data(&root);

        // println!("root_node: {:#?}", &root_node);
        // println!("bone_data_map: {:#?}", &model.bone_data_map.borrow());

        let mut model_animation = ModelAnimation {
            model: model.clone(),
            duration,
            ticks_per_second,
            root_node,
            node_animations: vec![].into(),
            bone_data_map: model.borrow().bone_data_map.clone(),
            global_inverse_transform
        };

        model_animation.read_channel_node_animations(&scene.animations[0]);
        model_animation
    }

    /// Converts scene.aiNode tree to local NodeData tree. Converting all the transforms to column major form.
    fn read_hierarchy_data(source: &Rc<Node>) -> NodeData {

        let mut node_data = NodeData {
            name: source.name.clone(),
            //transformation: convert_to_mat4(&source.transformation),
            transformation: source.transformation.clone(),
            children: vec![],
            meshes: source.meshes.clone(),
        };

        // println!("NodeData name: {}\n transform: {:?}\n", &node_data.name, &node_data.transformation);
        println!("NodeData: {} meshes: {:?}", &node_data.name, &source.meshes);

        for child in source.children.borrow().iter() {
            let node = ModelAnimation::read_hierarchy_data(child);
            node_data.children.push(node);
        }
        node_data
    }

    /// converts channel vec of Russimp::NodeAnims into vec of NodeAnimation
    fn read_channel_node_animations(&mut self, animation: &Animation) {
        for channel in &animation.channels {
            let node_animation = NodeAnimation::new(&channel.name.clone(), &channel);
            self.node_animations.borrow_mut().push(node_animation);
        }
    }
}

