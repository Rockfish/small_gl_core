use crate::transform::Transform;
use glam::{Mat4, Quat, Vec3};
use russimp::animation::NodeAnim;

#[derive(Debug, Clone)]
pub struct KeyPosition {
    pub position: Vec3,
    pub time_stamp: f32,
}

#[derive(Debug, Clone)]
pub struct KeyRotation {
    pub orientation: Quat,
    pub time_stamp: f32,
}

#[derive(Debug, Clone)]
pub struct KeyScale {
    pub scale: Vec3,
    pub time_stamp: f32,
}

#[derive(Debug, Clone)]
pub struct NodeAnimation {
    pub name: String,
    pub positions: Vec<KeyPosition>,
    pub rotations: Vec<KeyRotation>,
    pub scales: Vec<KeyScale>,
    // pub post_state: u32,
    // pub pre_state: u32,
}

#[derive(Debug, Clone)]
pub struct BoneData {
    pub name: String,
    pub bone_index: i32, // index connecting mesh bone_id array to transform in shader final_transform array
    pub offset: Mat4,    // offset from bone's parent
    pub offset_transform: Transform,
}

impl BoneData {
    pub fn new(name: impl Into<String>, id: i32, offset: Mat4) -> Self {
        BoneData {
            name: name.into(),
            bone_index: id,
            offset: offset.clone(),
            offset_transform: Transform::from_matrix(offset),
        }
    }
}

impl NodeAnimation {
    pub fn new(name: impl Into<String>, channel: &NodeAnim) -> Self {
        let positions: Vec<KeyPosition> = channel.position_keys.iter().map(|key| key.into()).collect();

        let rotations: Vec<KeyRotation> = channel.rotation_keys.iter().map(|key| key.into()).collect();

        let scales: Vec<KeyScale> = channel.scaling_keys.iter().map(|key| key.into()).collect();

        let name = name.into();
        println!("NodeAnimation: {}", &name);

        NodeAnimation {
            name,
            positions,
            rotations,
            scales,
        }
    }

    pub fn get_animation_transform_matrix(&self, animation_time: f32) -> Mat4 {
        let translation = Mat4::from_translation(self.interpolate_position(animation_time));
        let rotation = Mat4::from_quat(self.interpolate_rotation(animation_time));
        let scale = Mat4::from_scale(self.interpolate_scaling(animation_time));
        translation * rotation * scale
    }

    pub fn get_animation_transform(&self, animation_time: f32) -> Transform {
        Transform {
            translation: self.interpolate_position(animation_time),
            rotation: self.interpolate_rotation(animation_time),
            scale: self.interpolate_scaling(animation_time),
        }
    }

    fn interpolate_position(&self, animation_time: f32) -> Vec3 {
        if self.positions.len() == 1 {
            return self.positions[0].position;
        }

        let p0_index = self.get_position_index(animation_time);
        let p1_index = p0_index + 1;

        let scale_factor = self.get_scale_factor(
            self.positions[p0_index].time_stamp,
            self.positions[p1_index].time_stamp,
            animation_time,
        );

        let final_position = self.positions[p0_index]
            .position
            .lerp(self.positions[p1_index].position, scale_factor);

        final_position
    }

    fn interpolate_rotation(&self, animation_time: f32) -> Quat {
        if self.rotations.len() == 1 {
            let rotation = self.rotations[0].orientation.normalize();
            return rotation;
        }

        let p0_index = self.get_rotation_index(animation_time);
        let p1_index = p0_index + 1;

        let scale_factor = self.get_scale_factor(
            self.rotations[p0_index].time_stamp,
            self.rotations[p1_index].time_stamp,
            animation_time,
        );

        let final_rotation = self.rotations[p0_index]
            .orientation
            .slerp(self.rotations[p1_index].orientation, scale_factor);

        final_rotation
    }

    fn interpolate_scaling(&self, animation_time: f32) -> Vec3 {
        if self.scales.len() == 1 {
            return self.scales[0].scale;
        }

        let p0_index = self.get_scale_index(animation_time);
        let p1_index = p0_index + 1;

        let scale_factor = self.get_scale_factor(self.scales[p0_index].time_stamp, self.scales[p1_index].time_stamp, animation_time);

        let final_scale = self.scales[p0_index].scale.lerp(self.scales[p1_index].scale, scale_factor);

        final_scale
    }

    fn get_position_index(&self, animation_time: f32) -> usize {
        for index in 0..self.positions.len() - 1 {
            if animation_time < self.positions[index + 1].time_stamp {
                return index;
            }
        }
        panic!("animation time out of bounds");
    }

    fn get_rotation_index(&self, animation_time: f32) -> usize {
        for index in 0..self.rotations.len() - 1 {
            if animation_time < self.rotations[index + 1].time_stamp {
                return index;
            }
        }
        panic!("animation time out of bounds");
    }

    fn get_scale_index(&self, animation_time: f32) -> usize {
        for index in 0..self.scales.len() - 1 {
            if animation_time < self.scales[index + 1].time_stamp {
                return index;
            }
        }
        panic!("animation time out of bounds");
    }

    fn get_scale_factor(&self, last_timestamp: f32, next_timestamp: f32, animation_time: f32) -> f32 {
        let mid_way_length = animation_time - last_timestamp;
        let frames_diff = next_timestamp - last_timestamp;
        mid_way_length / frames_diff
    }
}
