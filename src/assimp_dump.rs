use russimp::animation::Animation;
use russimp::sys::aiScene;
use std::ptr::slice_from_raw_parts;

// fn load_model(scene: &AssimpScene) -> Result<(), Error> {
//     let option_ai_scene = unsafe { scene.assimp_scene.as_ref() };
//     match option_ai_scene {
//         None => Err(SceneError("Error getting scene".to_string())),
//         Some(ai_scene) => {
//             walk_node(ai_scene.mRootNode, ai_scene);
//             walk_animation_nodes(ai_scene);
//         }
//     }
//     Ok(())
// }
//

fn walk_animation_nodes(scene: &aiScene) {
    let slice = slice_from_raw_parts(scene.mAnimations, scene.mNumAnimations as usize);
    match unsafe { slice.as_ref() } {
        None => {}
        Some(animations) => {
            for i in 0..animations.len() {
                if unsafe { (*animations[i]).mName.length } > 0 {
                    let name: String = unsafe { (*animations[i]).mName.into() };
                    println!("animation name: {:?}", name);
                }
                let animation: Animation = unsafe { (&(*animations[i])).into() };
                // println!("animation: {:?}", animation);
                for node in animation.channels {
                    println!("NodeAnim: {:?}", node.name)
                }
                println!();
                for morph in animation.morph_mesh_channels {
                    println!("MeshMorphAnim: {:?}", morph.name)
                }
                println!();
                for mesh in animation.mesh_channels {
                    println!("MeshAnim: {:?}", mesh.name)
                }
                println!();
            }
        }
    }
}
