use crate::model::{BoneName, Model};
use crate::node_animation::NodeAnimation;
use crate::transform::Transform;
use crate::utils::HashMap;
use glam::Mat4;
use russimp::animation::Animation;
use russimp::node::Node;
use russimp::scene::Scene;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct NodeData {
    pub name: Rc<str>,
    pub transform_matrix: Mat4,
    pub transform: Transform,
    pub children: Vec<NodeData>,
    pub meshes: Rc<Vec<u32>>,
}

impl NodeData {
    pub fn new() -> Self {
        NodeData {
            name: Rc::from(""),
            transform_matrix: Mat4::IDENTITY,
            transform: Transform::IDENTITY,
            children: vec![],
            meshes: Rc::new(vec![]),
        }
    }
}

impl Default for NodeData {
    fn default() -> Self {
        NodeData {
            name: Rc::from(""),
            transform_matrix: Mat4::IDENTITY,
            transform: Transform::IDENTITY,
            children: vec![],
            meshes: Rc::new(vec![]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoneData {
    pub name: Rc<str>,
    pub bone_index: i32, // index connecting mesh bone_id array to transform in shader final_transform array
    pub offset_transform: Transform,
}

impl BoneData {
    pub fn new(name: &str, id: i32, offset: Mat4) -> Self {
        BoneData {
            name: name.into(),
            bone_index: id,
            offset_transform: Transform::from_matrix(offset),
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
            global_inverse_transform,
        };

        model_animation.read_channel_node_animations(&scene.animations[0]);
        model_animation
    }

    /// Converts scene.aiNode tree to local NodeData tree. Converting all the transforms to column major form.
    fn read_hierarchy_data(source: &Rc<Node>) -> NodeData {
        let mut node_data = NodeData {
            name: Rc::from(source.name.as_str()),
            //transformation: convert_to_mat4(&source.transformation),
            transform_matrix: source.transformation.clone(),
            transform: Transform::from_matrix(source.transformation.clone()),
            children: vec![],
            meshes: Rc::from(source.meshes.clone()),
        };

        // println!("NodeData: {} meshes: {:?}", &node_data.name, &source.meshes);

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
