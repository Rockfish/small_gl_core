use glam::{Mat4, quat, Quat, vec3, Vec3, Vec4, vec4};
use russimp::animation::{QuatKey, VectorKey};
use russimp::Matrix4x4;
use crate::node_animation::{KeyPosition, KeyRotation, KeyScale};


// Converts from row major matrix to column major matrix
pub fn convert_to_mat4(m: &Matrix4x4) -> Mat4 {
    //the a,b,c,d in assimp Matrix4x4 is the row ; the 1,2,3,4 is the column
    Mat4::from_cols(
        vec4(m.a1, m.b1, m.c1, m.d1), // m00, m01, m02, m03
        vec4(m.a2, m.b2, m.c2, m.d2), // m10, m11, m12, m13
        vec4(m.a3, m.b3, m.c3, m.d3), // m20, m21, m22, m23
        vec4(m.a4, m.b4, m.c4, m.d4), // m30, m31, m32, m33
    )
}

pub fn convert_matrix(mx: &Matrix4x4) -> Mat4 {
    Mat4::from_cols_slice(&[
        mx.a1, mx.b1, mx.c1, mx.d1,
        mx.a2, mx.b2, mx.c2, mx.d2,
        mx.a3, mx.b3, mx.c3, mx.d3,
        mx.a4, mx.b4, mx.c4, mx.d4,
    ])
}

const fn new_mat4(
    m00: f32,
    m01: f32,
    m02: f32,
    m03: f32,
    m10: f32,
    m11: f32,
    m12: f32,
    m13: f32,
    m20: f32,
    m21: f32,
    m22: f32,
    m23: f32,
    m30: f32,
    m31: f32,
    m32: f32,
    m33: f32,
) -> Mat4 {
    Mat4::from_cols(
        Vec4::new(m00, m01, m02, m03),
        Vec4::new(m10, m11, m12, m13),
        Vec4::new(m20, m21, m22, m23),
        Vec4::new(m30, m31, m32, m33),
    )
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
            orientation: Quat::from_xyzw(quad_key.value.x, quad_key.value.y, quad_key.value.z, quad_key.value.w),
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
