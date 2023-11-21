use std::cell::RefCell;
use std::rc::Rc;
use glam::Mat4;
use crate::animation::{ModelAnimation, NodeData};

struct Animator {
    current_time: f32,
    delta_time: f32,
    current_animation: Rc<RefCell<ModelAnimation>>,
    pub final_bone_matrices: Vec<Mat4>,
}

impl Animator {

    pub fn new(animation: &Rc<RefCell<ModelAnimation>>) -> Self {
        let mut final_bone_matrices = Vec::with_capacity(100);
        for _i in 0..100 {
            final_bone_matrices.push(Mat4::IDENTITY);
        }

        Animator {
            current_time: 0.0,
            delta_time: 0.0,
            current_animation: animation.clone(),
            final_bone_matrices,
        }
    }

    pub fn update_animation(&mut self, delta_time: f32) {
        self.delta_time = delta_time;
        self.current_time += self.current_animation.borrow().ticks_per_second * delta_time;
        self.current_time = self.current_time % self.current_animation.borrow().duration;
        let animation = &self.current_animation.clone();
        let root_node = &animation.borrow().root_node;
        self.calculate_bone_transform(root_node, Mat4::IDENTITY);
    }

    pub fn play_animation(&mut self, animation: &Rc<RefCell<ModelAnimation>>) {
        self.current_animation = animation.clone();
        self.current_time = 0.0;
    }

    pub fn calculate_bone_transform(&mut self, node: &NodeData, parent_transform: Mat4) {

        let mut node_transform = node.transformation.clone();

        if let Some(bone) = self.current_animation.borrow_mut().bones.iter_mut().find(|b| b.name == node.name) {
            bone.update(self.current_time);
            node_transform = bone.local_transform.clone();
        }

        let global_transformation = parent_transform * node_transform;

        if let Some(bone_info) = self.current_animation.borrow().bone_info_map.borrow().get(&node.name) {
            let index = bone_info.id as usize;
            self.final_bone_matrices[index] = global_transformation * bone_info.offset;
        }

        for child in node.children.iter() {
            self.calculate_bone_transform(child, global_transformation);
        }
    }

}