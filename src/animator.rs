use crate::hash_map::HashMap;
use crate::model_animation::{BoneData, BoneName, ModelAnimation, NodeData};
use crate::node_animation::NodeAnimation;
use crate::transform::Transform;
use glam::Mat4;
use russimp::node::Node;
use russimp::scene::Scene;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum AnimationRepeat {
    Once,
    Count(u32),
    Forever,
}

#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub start_tick: f32,
    pub end_tick: f32,
    pub repeat: AnimationRepeat,
}

impl AnimationClip {
    pub fn new(start_tick: f32, end_tick: f32, repeat: AnimationRepeat) -> Self {
        AnimationClip {
            start_tick,
            end_tick,
            repeat,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayingAnimation {
    pub animation_clip: Rc<AnimationClip>,
    pub current_tick: f32,
    pub ticks_per_second: f32,
    pub repeat_completions: u32,
}

impl PlayingAnimation {
    pub fn update(&mut self, delta_time: f32) {
        if self.current_tick < 0.0 {
            self.current_tick = self.animation_clip.start_tick;
        }

        self.current_tick += self.ticks_per_second * delta_time;

        if self.current_tick > self.animation_clip.end_tick {
            match self.animation_clip.repeat {
                AnimationRepeat::Once => {
                    self.current_tick = self.animation_clip.end_tick;
                }
                AnimationRepeat::Count(_) => {}
                AnimationRepeat::Forever => {
                    self.current_tick = self.animation_clip.start_tick;
                }
            }
            // in ticks
        }
    }
}

/// An animation that is being faded out as part of a transition (from Bevy)
#[derive(Debug, Clone)]
pub struct AnimationTransition {
    /// The current weight. Starts at 1.0 and goes to 0.0 during the fade-out.
    current_weight: f32,
    /// How much to decrease `current_weight` per second
    weight_decline_per_sec: f32,
    /// The animation that is being faded out
    animation: PlayingAnimation,
}

#[derive(Debug, Clone)]
pub struct NodeTransform {
    transform: Transform,
    meshes: Rc<Vec<u32>>,
}

impl NodeTransform {
    pub fn new(transform: Transform, meshes_vec: &Rc<Vec<u32>>) -> Self {
        NodeTransform {
            transform: transform.clone(),
            meshes: meshes_vec.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Animator {
    pub root_node: NodeData,
    pub global_inverse_transform: Mat4,
    pub bone_data_map: RefCell<HashMap<BoneName, BoneData>>,

    pub model_animation: ModelAnimation, // maybe should be vec?

    pub current_animation: PlayingAnimation,
    pub transitions: RefCell<Vec<AnimationTransition>>,

    pub node_transforms: RefCell<HashMap<Rc<str>, NodeTransform>>,

    pub final_bone_matrices: RefCell<Vec<Mat4>>,
    pub final_node_matrices: RefCell<Vec<Mat4>>,
}

impl Animator {
    pub fn new(scene: &Scene, bone_data_map: RefCell<HashMap<BoneName, BoneData>>) -> Self {
        let root = scene.root.as_ref().unwrap().clone();
        let global_inverse_transform = root.transformation.inverse();
        let root_node = read_hierarchy_data(&root);

        let model_animation = ModelAnimation::new(&scene);

        let mut final_bone_matrices = Vec::with_capacity(100);
        let mut final_node_matrices = Vec::with_capacity(50);

        for i in 0..100 {
            final_bone_matrices.push(Mat4::IDENTITY);
            if i < 50 {
                final_node_matrices.push(Mat4::IDENTITY);
            }
        }

        let animation_clip = AnimationClip {
            start_tick: 0.0,
            end_tick: model_animation.duration,
            repeat: AnimationRepeat::Forever,
        };

        let current_animation = PlayingAnimation {
            animation_clip: Rc::new(animation_clip),
            current_tick: -1.0,
            ticks_per_second: model_animation.ticks_per_second,
            repeat_completions: 0,
        };

        Animator {
            root_node,
            global_inverse_transform,
            bone_data_map,
            model_animation,
            current_animation,
            transitions: vec![].into(),
            node_transforms: HashMap::new().into(),
            final_bone_matrices: final_bone_matrices.into(),
            final_node_matrices: final_node_matrices.into(),
        }
    }

    pub fn play_clip(&mut self, clip: &Rc<AnimationClip>) {
        self.current_animation = PlayingAnimation {
            animation_clip: clip.clone(),
            current_tick: -1.0,
            ticks_per_second: self.model_animation.ticks_per_second,
            repeat_completions: 0,
        }
    }

    pub fn play_clip_with_transition(&mut self, clip: &Rc<AnimationClip>, transition_duration: Duration) {
        let mut animation = PlayingAnimation {
            animation_clip: clip.clone(),
            current_tick: -1.0,
            ticks_per_second: self.model_animation.ticks_per_second,
            repeat_completions: 0,
        };

        std::mem::swap(&mut animation, &mut self.current_animation);

        let transition = AnimationTransition {
            current_weight: 1.0,
            weight_decline_per_sec: 1.0 / transition_duration.as_secs_f32(),
            animation,
        };

        self.transitions.borrow_mut().push(transition);
    }

    pub fn update_animation(&mut self, delta_time: f32) {
        self.current_animation.update(delta_time);
        self.update_transitions(delta_time);
        self.update_node_map(delta_time);
        self.update_final_transforms();
    }

    fn update_node_map(&mut self, delta_time: f32) {
        self.node_transforms.borrow_mut().clear();

        let mut transitions = self.transitions.borrow_mut();
        let mut node_map = self.node_transforms.borrow_mut();
        let node_animations = self.model_animation.node_animations.borrow();

        let inverse_transform = Transform::from_matrix(self.global_inverse_transform.clone());

        // First for current animation at weight 1.0
        calculate_transform_maps(
            &self.root_node,
            &node_animations,
            &mut node_map,
            inverse_transform,
            self.current_animation.current_tick,
            1.0,
        );

        for transition in transitions.iter_mut() {
            transition.animation.update(delta_time);
            calculate_transform_maps(
                &self.root_node,
                &node_animations,
                &mut node_map,
                inverse_transform,
                transition.animation.current_tick,
                transition.current_weight,
            );
        }
    }

    fn update_transitions(&mut self, delta_time: f32) {
        self.transitions.borrow_mut().retain_mut(|animation| {
            animation.current_weight -= animation.weight_decline_per_sec * delta_time;
            animation.current_weight > 0.0
        })
    }

    fn update_final_transforms(&self) {
        let bone_data_map = self.bone_data_map.borrow();

        let mut final_bones = self.final_bone_matrices.borrow_mut();
        let mut final_node = self.final_node_matrices.borrow_mut();

        for (node_name, node_transform) in self.node_transforms.borrow_mut().iter() {
            if let Some(bone_data) = bone_data_map.get(node_name.deref()) {
                let index = bone_data.bone_index as usize;
                let transform_matrix = node_transform.transform.mul_transform(bone_data.offset_transform).compute_matrix();
                final_bones[index] = transform_matrix;
            }

            for mesh_index in node_transform.meshes.iter() {
                final_node[*mesh_index as usize] = node_transform.transform.compute_matrix();
            }
        }
    }
}

/// Converts scene Node tree to local NodeData tree. Converting all the transforms to column major form.
fn read_hierarchy_data(source: &Rc<Node>) -> NodeData {
    let mut node_data = NodeData {
        name: Rc::from(source.name.as_str()),
        transform: Transform::from_matrix(source.transformation.clone()),
        children: vec![],
        meshes: Rc::from(source.meshes.clone()),
    };

    // println!("NodeData: {} meshes: {:?}", &node_data.name, &source.meshes);

    for child in source.children.borrow().iter() {
        let node = read_hierarchy_data(child);
        node_data.children.push(node);
    }
    node_data
}

pub fn calculate_transform_maps(
    node_data: &NodeData,
    node_animations: &Ref<Vec<NodeAnimation>>,
    node_map: &mut RefMut<HashMap<Rc<str>, NodeTransform>>,
    parent_transform: Transform,
    current_tick: f32,
    weight: f32,
) {
    let global_transformation = calculate_transform(node_data, node_animations, node_map, parent_transform, current_tick, weight);

    for child_node in node_data.children.iter() {
        calculate_transform_maps(child_node, node_animations, node_map, global_transformation, current_tick, weight);
    }
}

fn calculate_transform(
    node_data: &NodeData,
    node_animations: &Ref<Vec<NodeAnimation>>,
    node_map: &mut RefMut<HashMap<Rc<str>, NodeTransform>>,
    parent_transform: Transform,
    current_tick: f32,
    weight: f32,
) -> Transform {
    let some_node_animation = node_animations.iter().find(|node_anim| node_anim.name == node_data.name);

    let global_transform = match some_node_animation {
        Some(node_animation) => {
            let node_transform = node_animation.get_animation_transform(current_tick);
            parent_transform.mul_transform(node_transform)
        }
        None => parent_transform.mul_transform(*&node_data.transform),
    };

    node_map
        .entry_ref(node_data.name.as_ref())
        .and_modify(|n| {
            n.transform = n.transform.mul_transform_weighted(global_transform, weight);
        })
        .or_insert(NodeTransform::new(global_transform, &node_data.meshes));

    global_transform
}
