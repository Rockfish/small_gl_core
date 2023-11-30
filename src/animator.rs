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
            current_time: 0.0,
            delta_time: 0.0,
            current_animation: animation.clone(),
            final_bone_matrices: final_bone_matrices.into(),
        }
    }

    pub fn update_animation(&mut self, delta_time: f32) {
        self.delta_time = delta_time;
        self.current_time += self.current_animation.borrow().ticks_per_second * delta_time;
        self.current_time = self.current_time % self.current_animation.borrow().duration;

        // println!("animation current_time: {}", self.current_time);

        let animation = &self.current_animation.clone();
        let root_node = &animation.borrow().root_node;

        self.calculate_bone_transform(root_node, Mat4::IDENTITY);

        // println!("animation update completed.");
    }

    pub fn play_animation(&mut self, animation: &Rc<RefCell<ModelAnimation>>) {
        self.current_animation = animation.clone();
        self.current_time = 0.0;
    }

    pub fn calculate_bone_transform(&self, node_data: &NodeData, parent_transform: Mat4) {

        let mut global_transformation: Mat4 = Mat4::IDENTITY;

        let mut node_transform = &node_data.transformation;

        {
            let current_animation = self.current_animation.borrow();
            let mut bone_animations = current_animation.bone_animations.borrow_mut();

            let some_animation = bone_animations.iter_mut().find(|bone_anim| bone_anim.name == node_data.name);

            if let Some(bone_animation) = some_animation {
                bone_animation.update(self.current_time);
                node_transform = &bone_animation.local_transform;
            }

            global_transformation = parent_transform * *node_transform;

            // println!("node_name: {}\nparent_transform: {:?}\nnode_transform: {:?}\nglobal_transform: {:?}\n", &node_data.name, &parent_transform, &transform, &global_transformation);

            if let Some(bone_data) = self
                .current_animation
                .borrow()
                .bone_data_map
                .borrow()
                .get(&node_data.name)
            {
                let index = bone_data.bone_index as usize;
                let mut final_bones = self.final_bone_matrices.borrow_mut();
                // final_bones[index] = current_animation.global_inverse_transform * global_transformation * bone_data.offset;
                final_bones[index] = global_transformation * bone_data.offset;
            }
        }

        for child in node_data.children.iter() {
            self.calculate_bone_transform(child, global_transformation);
        }
    }
}
