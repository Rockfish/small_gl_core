use crate::model_animation::{ModelAnimation, NodeData};
use glam::Mat4;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use crate::transform::Transform;
use crate::utils::HashMap;

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
    pub repeat: AnimationRepeat
}

impl AnimationClip {
    pub fn new(name: impl Into<String>, start_tick: f32, end_tick:f32, repeat: AnimationRepeat) -> Self {
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
            self.current_tick = self.animation_clip.start_tick; // in ticks
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
    meshes: Option<Vec<u32>>,
}

impl Default for NodeTransform {
    fn default() -> Self {
        NodeTransform {
            transform: Transform::IDENTITY,
            meshes: None,
        }
    }
}

pub struct Animator {
    model_animation: Rc<RefCell<ModelAnimation>>,
    
    current_animation: PlayingAnimation,
    transitions: Vec<AnimationTransition>,

    node_transforms: HashMap<String, NodeTransform>,

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
            animation_clip:  Rc::new(animation_clip),
            current_tick: -1.0,
            ticks_per_second: animation.borrow().ticks_per_second,
            repeat_completions: 0,
        };

        Animator {
            model_animation: animation.clone(),
            final_bone_matrices: final_bone_matrices.into(),
            current_animation,
            transitions: vec![],
            node_transforms: HashMap::new(),
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

        self.transitions.push(transition);
    }

    pub fn update_animation(&mut self, delta_time: f32) {

        self.current_animation.update(delta_time);

        let animation = &self.model_animation.clone();
        let root_node = &animation.borrow().root_node;

        self.node_transforms.clear();

        // using matrices
        // self.calculate_transform_one_pass(root_node, self.model_animation.borrow().global_inverse_transform.clone());

        // First for current animation at weight 1.0
        let inverse_transform = Transform::from_matrix(self.model_animation.borrow().global_inverse_transform.clone());
        self.calculate_transform_maps(root_node, inverse_transform, 1.0);

        // Then for transitions with their weights
        for transition in self.transitions.clone() {
            self.calculate_transform_maps(root_node, inverse_transform, transition.current_weight);
        }

        // set final for the shader
        self.set_final_transforms();
    }

    // pub fn calculate_transform_one_pass(&self, node_data: &NodeData, parent_transform: Mat4) {
    //     let global_transformation = self.calculate_global_transform(node_data, parent_transform);
    //
    //     for child in node_data.children.iter() {
    //         self.calculate_transform_one_pass(child, global_transformation);
    //     }
    // }
    //
    // // notes for blending animation -
    // // should calculate first since the transforms are propagated down the tree
    // // Later after all the values have been calculated for the nodes by the current tick time for the current anim sequence
    // // store the values in a map<NodeData, transform> for each sequence
    // // Then do a weighted merge maps
    // // and when doing the merge calculate and store the final_bone transforms, and store the mesh transforms, for the shader to apply
    // fn calculate_global_transform(&self, node_data: &NodeData, parent_transform: Mat4) -> Mat4 {
    //     let current_animation = self.model_animation.borrow();
    //     let bone_data_map = current_animation.bone_data_map.borrow();
    //     let mut node_animations = current_animation.node_animations.borrow_mut();
    //
    //     let some_animation = node_animations.iter().find(|node_anim| node_anim.name == node_data.name);
    //
    //     let global_transformation = match some_animation {
    //         Some(node_animation) => parent_transform * node_animation.get_animation_transform_matrix(self.current_animation.current_tick),
    //         None => parent_transform * *&node_data.transform_matrix,
    //     };
    //
    //     // println!("node_name: {} global_transform: {:?}\n", &node_data.name, &global_transformation);
    //
    //     if let Some(bone_data) = bone_data_map.get(&node_data.name) {
    //         let index = bone_data.bone_index as usize;
    //         let mut final_bones = self.final_bone_matrices.borrow_mut();
    //         final_bones[index] = global_transformation * bone_data.offset;
    //     }
    //
    //     for mesh_index in node_data.meshes.iter() {
    //         self.model_animation.borrow().model.borrow_mut().meshes.borrow_mut()[*mesh_index as usize].node_transform =
    //             global_transformation;
    //     }
    //
    //     global_transformation
    // }

    pub fn calculate_transform_maps(&mut self, node_data: &NodeData, parent_transform: Transform, weight: f32) {
        let global_transformation = self.calculate_transform(node_data, parent_transform, weight);

        for child in node_data.children.iter() {
            self.calculate_transform_maps(child, global_transformation, weight);
        }
    }

    // sets transform maps
    fn calculate_transform(&mut self, node_data: &NodeData, parent_transform: Transform, weight: f32) -> Transform {

        let current_animation = self.model_animation.borrow();
        let mut node_animations = current_animation.node_animations.borrow_mut();

        let some_animation = node_animations.iter().find(|node_anim| node_anim.name == node_data.name);

        let global_transform = match some_animation {
            Some(node_animation) => {
                let node_transform = node_animation.get_animation_transform(self.current_animation.current_tick);
                parent_transform.mul_transform(node_transform)
            },
            None => {
                parent_transform.mul_transform(*&node_data.transform)
            },
        };

        let node_transform = self.node_transforms.entry_ref(&node_data.name).or_insert(NodeTransform::default());

        // todo: replace with weighted
        (*node_transform).transform = node_transform.transform.mul_transform(global_transform);

        if !node_data.meshes.is_empty() {
            (*node_transform).meshes = Some(node_data.meshes.clone());
        }

        global_transform
    }

    fn set_final_transforms(&self) {
        let current_animation = self.model_animation.borrow();
        let bone_data_map = current_animation.bone_data_map.borrow();

        for (node_name, node_transform) in &self.node_transforms {

            if let Some(bone_data) = bone_data_map.get(node_name) {
                let index = bone_data.bone_index as usize;
                let mut final_bones = self.final_bone_matrices.borrow_mut();
                let transform_matrix = node_transform.transform.mul_transform(bone_data.offset_transform).compute_matrix();
                final_bones[index] = transform_matrix;
            }

            if let Some(meshes) = &node_transform.meshes {
                for mesh_index in meshes.iter() {
                    self.model_animation.borrow().model.borrow_mut().meshes.borrow_mut()[*mesh_index as usize].node_transform =
                        node_transform.transform.compute_matrix();
                }
            }
        }
    }
}
