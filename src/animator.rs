use crate::model_animation::{ModelAnimation, NodeData};
use glam::Mat4;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Animator {
    current_time: f32,
    delta_time: f32,
    current_animation: Rc<RefCell<ModelAnimation>>,
    pub final_bone_matrices: RefCell<Vec<Mat4>>,
}

impl Animator {
    pub fn new(animation: &Rc<RefCell<ModelAnimation>>) -> Self {
        let mut final_bone_matrices = Vec::with_capacity(100);

        for _i in 0..100 {
            final_bone_matrices.push(Mat4::IDENTITY);
        }

        Animator {
            current_time: -1.0,
            delta_time: 0.0,
            current_animation: animation.clone(),
            final_bone_matrices: final_bone_matrices.into(),
        }
    }

    pub fn update_animation(&mut self, delta_time: f32) {
        self.delta_time = delta_time;

        if self.current_time < 0.0 {
            self.current_time = 0.0;
        }

        self.current_time += self.current_animation.borrow().ticks_per_second * delta_time;
        self.current_time = self.current_time % self.current_animation.borrow().duration;

        // println!("animation current_time: {}", self.current_time);

        let animation = &self.current_animation.clone();
        let root_node = &animation.borrow().root_node;

        // self.calculate_bone_transform(root_node, Mat4::IDENTITY);
        self.calculate_bone_transforms(root_node, self.current_animation.borrow().global_inverse_transform.clone());

        // println!("animation update completed.");
    }

    pub fn update_animation_sequence(&mut self, offset: f32, duration: f32, delta_time: f32) {
        self.delta_time = delta_time;

        if self.current_time < 0.0 {
            self.current_time = offset; // in ticks
        }

        self.current_time += self.current_animation.borrow().ticks_per_second * delta_time;

        if self.current_time > duration {
            self.current_time = offset; // in ticks
        }

        let animation = &self.current_animation.clone();
        let root_node = &animation.borrow().root_node;

        self.calculate_bone_transforms(root_node, self.current_animation.borrow().global_inverse_transform.clone());
    }

    pub fn play_animation(&mut self, animation: &Rc<RefCell<ModelAnimation>>) {
        self.current_animation = animation.clone();
        self.current_time = 0.0;
    }

    pub fn calculate_bone_transforms(&self, node_data: &NodeData, parent_transform: Mat4) {
        let global_transformation = self.calculate_transform(node_data, parent_transform);

        for child in node_data.children.iter() {
            self.calculate_bone_transforms(child, global_transformation);
        }
    }

    // notes for blending animation -
    // should calculate first since the transforms are propagated down the tree
    // Later after all the values have been calculated for the nodes by the current tick time for the current anim sequence
    // store the values in a map<NodeData, transform> for each sequence
    // Then do a weighted merge maps
    // and when doing the merge calculate and store the final_bone transforms, and store the mesh transforms, for the shader to apply
    fn calculate_transform(&self, node_data: &NodeData, parent_transform: Mat4) -> Mat4 {
        let current_animation = self.current_animation.borrow();
        let bone_data_map = current_animation.bone_data_map.borrow();
        let mut node_animations = current_animation.node_animations.borrow_mut();

        let some_animation = node_animations.iter().find(|bone_anim| bone_anim.name == node_data.name);

        let global_transformation = match some_animation {
            Some(node_animation) => parent_transform * node_animation.get_animation_transform(self.current_time),
            None => parent_transform * *&node_data.transformation,
        };

        // println!("node_name: {} global_transform: {:?}\n", &node_data.name, &global_transformation);

        if let Some(bone_data) = bone_data_map.get(&node_data.name) {
            let index = bone_data.bone_index as usize;
            let mut final_bones = self.final_bone_matrices.borrow_mut();
            final_bones[index] = global_transformation * bone_data.offset;
        }

        for mesh_index in node_data.meshes.iter() {
            self.current_animation.borrow().model.borrow_mut().meshes.borrow_mut()[*mesh_index as usize].node_transform =
                global_transformation;
        }

        global_transformation
    }
}
