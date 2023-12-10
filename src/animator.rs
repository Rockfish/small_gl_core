use crate::model_animation::{ModelAnimation, NodeData};
use crate::node_animation::NodeAnimation;
use crate::transform::Transform;
use crate::utils::HashMap;
use glam::Mat4;
use std::cell::{Ref, RefCell, RefMut};
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
    pub name: String,
    pub start_tick: f32,
    pub end_tick: f32,
    pub repeat: AnimationRepeat,
}

impl AnimationClip {
    pub fn new(name: impl Into<String>, start_tick: f32, end_tick: f32, repeat: AnimationRepeat) -> Self {
        AnimationClip {
            name: name.into(),
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
struct AnimationTransition {
    /// The current weight. Starts at 1.0 and goes to 0.0 during the fade-out.
    current_weight: f32,
    /// How much to decrease `current_weight` per second
    weight_decline_per_sec: f32,
    /// The animation that is being faded out
    animation: PlayingAnimation,
}

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

pub struct Animator {
    model_animation: Rc<RefCell<ModelAnimation>>,

    delta_time: f32,
    current_animation: PlayingAnimation,
    transitions: RefCell<Vec<AnimationTransition>>,

    node_transforms: RefCell<HashMap<Rc<str>, NodeTransform>>,

    pub final_bone_matrices: RefCell<Vec<Mat4>>,
}

impl Animator {
    pub fn new(animation: &Rc<RefCell<ModelAnimation>>) -> Self {
        let mut final_bone_matrices = Vec::with_capacity(100);

        for _i in 0..100 {
            final_bone_matrices.push(Mat4::IDENTITY);
        }

        let animation_clip = AnimationClip {
            name: "Model".to_string(),
            start_tick: 0.0,
            end_tick: animation.borrow().duration,
            repeat: AnimationRepeat::Forever,
        };

        let current_animation = PlayingAnimation {
            animation_clip: Rc::new(animation_clip),
            current_tick: -1.0,
            ticks_per_second: animation.borrow().ticks_per_second,
            repeat_completions: 0,
        };

        Animator {
            model_animation: animation.clone(),
            final_bone_matrices: final_bone_matrices.into(),
            current_animation,
            transitions: vec![].into(),
            node_transforms: HashMap::new().into(),
            delta_time: 0.0,
        }
    }

    pub fn play_clip(&mut self, clip: &Rc<AnimationClip>) {
        self.current_animation = PlayingAnimation {
            animation_clip: clip.clone(),
            current_tick: -1.0,
            ticks_per_second: self.model_animation.borrow().ticks_per_second,
            repeat_completions: 0,
        }
    }

    pub fn play_clip_with_transition(&mut self, clip: &Rc<AnimationClip>, transition_duration: Duration) {
        let mut animation = PlayingAnimation {
            animation_clip: clip.clone(),
            current_tick: -1.0,
            ticks_per_second: self.model_animation.borrow().ticks_per_second,
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
        self.delta_time = delta_time;
        self.current_animation.update(delta_time);
        self.update_transitions();

        self.update_node_map(delta_time);

        // set final for the shader
        self.set_final_transforms();
    }

    fn update_node_map(&mut self, delta_time: f32) {
        let animation = &self.model_animation.clone();
        let root_node = &animation.borrow().root_node;
        self.node_transforms.borrow_mut().clear();
        let mut transitions = self.transitions.borrow_mut();
        let mut node_map = self.node_transforms.borrow_mut();
        let model_animation = self.model_animation.borrow();
        let node_animations = model_animation.node_animations.borrow();

        let inverse_transform = Transform::from_matrix(self.model_animation.borrow().global_inverse_transform.clone());

        // First for current animation at weight 1.0
        Animator::calculate_transform_maps(
            root_node,
            &node_animations,
            &mut node_map,
            inverse_transform,
            self.current_animation.current_tick,
            1.0,
        );

        for transition in transitions.iter_mut() {
            transition.animation.update(delta_time);
            Animator::calculate_transform_maps(
                root_node,
                &node_animations,
                &mut node_map,
                inverse_transform,
                transition.animation.current_tick,
                transition.current_weight,
            );
        }
    }

    pub fn calculate_transform_maps(
        node_data: &NodeData,
        node_animations: &Ref<Vec<NodeAnimation>>,
        node_map: &mut RefMut<HashMap<Rc<str>, NodeTransform>>,
        parent_transform: Transform,
        current_tick: f32,
        weight: f32,
    ) {
        let global_transformation =
            Animator::calculate_transform(node_data, node_animations, node_map, parent_transform, current_tick, weight);

        for child_node in node_data.children.iter() {
            Animator::calculate_transform_maps(child_node, node_animations, node_map, global_transformation, current_tick, weight);
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

    fn update_transitions(&mut self) {
        self.transitions.borrow_mut().retain_mut(|animation| {
            animation.current_weight -= animation.weight_decline_per_sec * self.delta_time;
            animation.current_weight > 0.0
        })
    }

    fn set_final_transforms(&self) {
        let current_animation = self.model_animation.borrow();
        let bone_data_map = current_animation.bone_data_map.borrow();

        for (node_name, node_transform) in self.node_transforms.borrow_mut().iter() {
            if let Some(bone_data) = bone_data_map.get(node_name.as_ref()) {
                let index = bone_data.bone_index as usize;
                let mut final_bones = self.final_bone_matrices.borrow_mut();
                let transform_matrix = node_transform.transform.mul_transform(bone_data.offset_transform).compute_matrix();
                final_bones[index] = transform_matrix;
            }

            for mesh_index in node_transform.meshes.iter() {
                self.model_animation.borrow().model.borrow_mut().meshes.borrow_mut()[*mesh_index as usize].node_transform =
                    node_transform.transform.compute_matrix();
            }
        }
    }
}
