use std::rc::Rc;
use glam::{Mat4, Quat, quat, Vec3, vec3};
use russimp::animation::{NodeAnim, QuatKey, VectorKey};

#[derive(Debug, Clone)]
pub struct KeyPosition {
    pub position: Vec3,
    time_stamp: f32,
}

#[derive(Debug, Clone)]
pub struct KeyRotation {
    pub orientation: Quat,
    time_stamp: f32,
}

#[derive(Debug, Clone)]
pub struct KeyScale {
    pub scale: Vec3,
    time_stamp: f32,
}

#[derive(Debug, Clone)]
pub struct Bone {
    pub id: i32,
    pub name: String,
    pub positions: Vec<KeyPosition>,
    pub rotations: Vec<KeyRotation>,
    pub scales: Vec<KeyScale>,
    pub local_transform: Mat4,
}

#[derive(Debug, Clone)]
pub struct BoneInfo {
    pub id: i32,
    pub offset: Mat4,
}

impl BoneInfo {
    pub fn new(id: i32, offset: Mat4) -> Self {
        BoneInfo { id, offset }
    }
}

impl Bone {
    pub fn new(name: impl Into<String>, id: i32, channel: &NodeAnim) -> Self {

        let positions: Vec<KeyPosition> = channel.position_keys.iter().map(|key| key.into()).collect();
        let rotations: Vec<KeyRotation> = channel.rotation_keys.iter().map(|key| key.into()).collect();
        let scales: Vec<KeyScale> = channel.position_keys.iter().map(|key| key.into()).collect();

        Bone {
            id,
            name: name.into(),
            positions,
            rotations,
            scales,
            local_transform: Default::default(),
        }
    }

    pub fn update(&mut self, animation_time: f32) {
        let translation = self.interpolate_position(animation_time);
        let rotation = self.interpolate_rotation(animation_time);
        let scale = self.interpolate_rotation(animation_time);
        self.local_transform = translation * rotation * scale;
    }

    fn interpolate_position(&self, animation_time: f32) -> Mat4 {
        if self.positions.len() == 1 {
            return Mat4::from_translation(self.positions[0].position);
        }
        let p0_index = self.get_position_index(animation_time);
        let p1_index = p0_index + 1;
        let scale_factor = self.get_scale_factor(self.positions[p0_index].time_stamp, self.positions[p1_index].time_stamp, animation_time);
        let final_position = self.positions[p0_index].position.lerp( self.positions[p1_index].position, scale_factor);
        Mat4::from_translation(final_position)
    }

    fn interpolate_rotation(&self, animation_time: f32) -> Mat4 {
        if self.rotations.len() == 1 {
            let rotation = self.rotations[0].orientation.normalize();
            return Mat4::from_quat(rotation);
        }
        let p0_index = self.get_rotation_index(animation_time);
        let p1_index = p0_index + 1;
        let scale_factor = self.get_scale_factor(self.rotations[p0_index].time_stamp, self.rotations[p1_index].time_stamp, animation_time);
        let final_rotation = self.rotations[p0_index].orientation.slerp(self.rotations[p1_index].orientation, scale_factor);
        Mat4::from_quat(final_rotation)
    }

    fn interpolate_scaling(&self, animation_time: f32) -> Mat4 {
        if self.scales.len() == 1 {
            return Mat4::from_scale(self.scales[0].scale);
        }
        let p0_index = self.get_scale_index(animation_time);
        let p1_index = p0_index + 1;
        let scale_factor = self.get_scale_factor(self.scales[p0_index].time_stamp, self.scales[p1_index].time_stamp, animation_time);
        let final_scale = self.scales[p0_index].scale.lerp( self.scales[p1_index].scale, scale_factor);
        Mat4::from_scale(final_scale)
    }

    // todo: double check that its returning the right index
    fn get_position_index(&self, animation_time: f32) -> usize {
        self.positions.iter().position(|key| key.time_stamp > animation_time).unwrap() - 1
    }

    fn get_rotation_index(&self, animation_time: f32) -> usize {
        self.rotations.iter().position(|key| key.time_stamp > animation_time).unwrap() - 1
    }

    fn get_scale_index(&self, animation_time: f32) -> usize {
        self.scales.iter().position(|key| key.time_stamp > animation_time).unwrap() - 1
    }

    fn get_scale_factor(&self, last_timestamp: f32, next_timestamp: f32, animation_time: f32) -> f32 {
        let mid_way_length = animation_time - last_timestamp;
        let frames_diff = next_timestamp - last_timestamp;
        mid_way_length / frames_diff
    }
}

impl From<&VectorKey> for KeyPosition {
    fn from(vector_key: &VectorKey) -> Self {
        KeyPosition {
            position: vec3(vector_key.value.x, vector_key.value.y, vector_key.value.z),
            time_stamp: vector_key.time as f32,
        }
    }
}

impl From<&QuatKey> for KeyRotation {
    fn from(quad_key: &QuatKey) -> Self {
        KeyRotation {
            orientation: quat(quad_key.value.x, quad_key.value.y, quad_key.value.z, quad_key.value.w),
            time_stamp: quad_key.time as f32,
        }
    }
}

impl From<&VectorKey> for KeyScale {
    fn from(vector_key: &VectorKey) -> Self {
        KeyScale {
            scale: vec3(vector_key.value.x, vector_key.value.y, vector_key.value.z),
            time_stamp: vector_key.time as f32,
        }
    }
}