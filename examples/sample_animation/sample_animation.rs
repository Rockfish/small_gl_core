#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]
#![allow(clippy::zero_ptr)]
#![allow(clippy::assign_op_pattern)]

// extern crate glfw;

mod cube_object;

use glam::*;
use glfw::{Action, Context, Key};
use image::ColorType;
use log::error;
use small_gl_core::animator::Animator;
use small_gl_core::camera::{Camera, CameraMovement};
use small_gl_core::gl;
use small_gl_core::gl::{GLint, GLsizei, GLuint, GLvoid};
use small_gl_core::model::{Model, ModelBuilder};
use small_gl_core::model_animation::ModelAnimation;
use small_gl_core::shader::Shader;
use small_gl_core::texture::TextureType;
use std::cell::RefCell;
use std::rc::Rc;

const SCR_WIDTH: f32 = 800.0;
const SCR_HEIGHT: f32 = 800.0;

// Lighting
const LIGHT_FACTOR: f32 = 1.0;
const NON_BLUE: f32 = 0.9;

const FLOOR_LIGHT_FACTOR: f32 = 0.35;
const FLOOR_NON_BLUE: f32 = 0.7;

// Struct for passing state between the window loop and the event handler.
struct State {
    camera: Camera,
    lightPos: Vec3,
    deltaTime: f32,
    lastFrame: f32,
    firstMouse: bool,
    lastX: f32,
    lastY: f32,
}

fn error_callback(err: glfw::Error, description: String) {
    error!("GLFW error {:?}: {:?}", err, description);
}

fn main() {
    let mut glfw = glfw::init(error_callback).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    // for Apple
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(SCR_WIDTH as u32, SCR_HEIGHT as u32, "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // Turn on all GLFW polling so that we can receive all WindowEvents
    window.set_all_polling(true);
    window.make_current();

    // Initialize glad: load all OpenGL function pointers
    // --------------------------------------------------
    gl::load(|e| glfw.get_proc_address_raw(e) as *const std::os::raw::c_void);

    let camera = Camera::camera_vec3(vec3(0.0, 40.0, 120.0));

    // Initialize the world state
    let mut state = State {
        camera,
        lightPos: vec3(1.2, 1.0, 2.0),
        deltaTime: 0.0,
        lastFrame: 0.0,
        firstMouse: true,
        lastX: SCR_WIDTH / 2.0,
        lastY: SCR_HEIGHT / 2.0,
    };

    // configure global opengl state
    // -----------------------------
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    let shader =
        Rc::new(
            Shader::new(
                "examples/sample_animation/anim_model.vert",
                "examples/sample_animation/anim_model.frag",
            )
            .unwrap(),
        );

    let shader =
        Rc::new(
            Shader::new(
                "examples/sample_animation/player_shader.vert",
                "examples/sample_animation/player_shader.frag",
            )
            .unwrap(),
        );

    let model_path = "examples/sample_animation/vampire/dancing_vampire.dae";
    // let model_path = "/Users/john/Dev_Rust/Dev/russimp_glam/models/GLTF2/round_wooden_table_01_4k/round_wooden_table_01_4k.gltf";
    // let model_path = "/Users/john/Dev_Rust/Dev/learn_opengl_with_rust/resources/objects/nanosuit/nanosuit.obj";
    // let model_path = "/Users/john/Dev_Assets/glTF-Sample-Models/2.0/CesiumMan/glTF/CesiumMan.gltf"; // works
    // let model_path = "/Users/john/Dev_Rust/Repos/OpenGL-Tutorials/LearnOpenGL/8.Guest Articles/2020/2.Skeletal Animation/resources/objects/vampire/dancing_vampire.dae";
    let model_path = "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Player.fbx";
    // let model_path = "/Users/john/Dev_Assets/animated-characters-3/Model/characterMedium.fbx";
    // let model_path = "/Users/john/Dev_Rust/Dev/alien_explorer/assets/models/alien.glb";
    // let cube = Cube::new("cube", shader.clone());
    // let model_path = "examples/sample_animation/source/cube_capoeira_martelo_cruzando.FBX.fbx"; // platform with martial arts guy
    // let model_path = "/Users/john/Dev_Rust/Repos/ogldev/Content/box.obj"; // no animations
    // let model_path = "/Users/john/Dev_Rust/Repos/OpenGL-Animation/Resources/res/model.dae"; // doesn't load
    // let model_path = "examples/sample_animation/colorful_cube/scene.gltf";  // small cube, doesn't animate
    // let model_path = "/Users/john/Dev_Rust/Dev/learn_opengl_with_rust/resources/objects/cyborg/cyborg.obj"; // not animated

    // let scene = AssimpScene::load_assimp_scene(model_path).unwrap();
    let scene = ModelBuilder::load_russimp_scene(model_path).unwrap();

    let dancing_model = ModelBuilder::new("model", shader.clone(), model_path)
        .add_texture("Player", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Player_D.tga") // Player model
        .add_texture("Player", TextureType::Specular, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Player_M.tga") // Player model
        .add_texture("Player", TextureType::Emissive, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Player_E.tga") // Player model
        .add_texture("Player", TextureType::Normals, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Player_NRM.tga") // Player model
        .add_texture("Gun", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Gun_D.tga") // Player model
        .add_texture("Gun", TextureType::Specular, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Gun_M.tga") // Player model
        .add_texture("Gun", TextureType::Emissive, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Gun_E.tga") // Player model
        .add_texture("Gun", TextureType::Normals, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Gun_NRM.tga") // Player model
        // .add_texture("characterMedium", TextureType::Diffuse, "/Users/john/Dev_Assets/animated-characters-3/Skins/humanFemaleA.png")  // characterMedium model
        // .add_texture("Box016", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box009", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box008", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box007", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box010", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box011", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box012", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box001", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box006", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box005", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box004", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box015", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box014", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box013", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box002", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Box003", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/container2.png") // capoeira
        // .add_texture("Cylinder001", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/small_gl_core/examples/sample_animation/animated_cube/AnimatedCube_BaseColor.png") // capoeira
        .build_with_scene(&scene)
        .unwrap();

    let dancing_model = Rc::new(RefCell::new(dancing_model));

    let dance_animation = Rc::new(RefCell::new(ModelAnimation::new(&scene, dancing_model.clone())));
    let mut animator = Animator::new(&dance_animation);

    // Lighting
    let lightDir: Vec3 = vec3(-0.8, 0.0, -1.0).normalize_or_zero();
    let playerLightDir: Vec3 = vec3(-1.0, -1.0, -1.0).normalize_or_zero();

    let lightColor: Vec3 = LIGHT_FACTOR * 1.0 * vec3(NON_BLUE * 0.406, NON_BLUE * 0.723, 1.0);
    // const lightColor: Vec3 = LIGHT_FACTOR * 1.0 * vec3(0.406, 0.723, 1.0);

    let floorLightColor: Vec3 = FLOOR_LIGHT_FACTOR * 1.0 * vec3(FLOOR_NON_BLUE * 0.406, FLOOR_NON_BLUE * 0.723, 1.0);
    let floorAmbientColor: Vec3 = FLOOR_LIGHT_FACTOR * 0.50 * vec3(FLOOR_NON_BLUE * 0.7, FLOOR_NON_BLUE * 0.7, 0.7);

    let ambientColor: Vec3 = LIGHT_FACTOR * 1.0 * vec3(NON_BLUE * 0.7, NON_BLUE * 0.7, 0.7);

    state.lastFrame = glfw.get_time() as f32;

    // render loop
    while !window.should_close() {
        let currentFrame = glfw.get_time() as f32;
        state.deltaTime = currentFrame - state.lastFrame;
        state.lastFrame = currentFrame;

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut state);
        }

        // println!("time: {}   delta: {}", state.lastFrame, state.deltaTime);
        // animator.update_animation(state.deltaTime);
        // animator.update_animation(0.01);

        // animation - duration: 294   ticks_per_second: 30

        let movement_duration = 20.0f32;

        // animator.update_animation_sequence(55.0, 130.0, state.deltaTime); // Idle
        // animator.update_animation_sequence(134.0, 154.0, state.deltaTime); // Forward running
        // animator.update_animation_sequence(159.0, 179.0, state.deltaTime); // Backwards running
        animator.update_animation_sequence(184.0, 204.0, state.deltaTime); // Right running
                                                                           // animator.update_animation_sequence(209.0, 229.0, state.deltaTime); // Left running
                                                                           // animator.update_animation_sequence(234.0, 293.0, state.deltaTime); // Dying

        unsafe {
            // render
            gl::ClearColor(0.05, 0.1, 0.05, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // be sure to activate shader when setting uniforms/drawing objects
            shader.use_shader();

            // view/projection transformations
            let projection = Mat4::perspective_rh_gl(state.camera.zoom.to_radians(), SCR_WIDTH / SCR_HEIGHT, 0.1, 1000.0);

            let view = state.camera.get_view_matrix();
            shader.set_mat4("projection", &projection);
            shader.set_mat4("view", &view);

            let final_bones = animator.final_bone_matrices.borrow_mut();

            for (i, bone_transform) in final_bones.iter().enumerate() {
                shader.set_mat4(format!("finalBonesMatrices[{}]", i).as_str(), &bone_transform);
            }

            let mut model = Mat4::IDENTITY;
            // model *= Mat4::from_rotation_x(-90.0f32.to_radians());
            model *= Mat4::from_translation(vec3(0.0, -10.4, -400.0));
            // model *= Mat4::from_scale(vec3(0.3, 0.3, 0.3));
            // let mut model = Mat4::from_translation(vec3(0.0, 5.0, 0.0));
            // model = model * Mat4::from_scale(vec3(15.0, 15.0, 15.0));
            model = model * Mat4::from_scale(vec3(1.0, 1.0, 1.0));

            shader.set_mat4("model", &model);

            shader.set_bool("useLight", true);
            shader.set_vec3("ambient", &ambientColor);

            shader.set_mat4("aimRot", &Mat4::IDENTITY);
            shader.set_mat4("lightSpaceMatrix", &Mat4::IDENTITY);

            dancing_model.borrow().render();
        }

        window.swap_buffers();
    }
}

//
// GLFW maps callbacks to events.
//
fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, state: &mut State) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::FramebufferSize(width, height) => {
            framebuffer_size_event(window, width, height);
        }
        glfw::WindowEvent::Key(Key::W, _, _, _) => {
            state.camera.process_keyboard(CameraMovement::Forward, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::S, _, _, _) => {
            state.camera.process_keyboard(CameraMovement::Backward, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::A, _, _, _) => {
            state.camera.process_keyboard(CameraMovement::Left, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::D, _, _, _) => {
            state.camera.process_keyboard(CameraMovement::Right, state.deltaTime);
        }
        glfw::WindowEvent::CursorPos(xpos, ypos) => mouse_handler(state, xpos, ypos),
        glfw::WindowEvent::Scroll(xoffset, ysoffset) => scroll_handler(state, xoffset, ysoffset),
        _evt => {
            // println!("WindowEvent: {:?}", evt);
        }
    }
}

// glfw: whenever the window size changed (by OS or user resize) this event fires.
// ---------------------------------------------------------------------------------------------
fn framebuffer_size_event(_window: &mut glfw::Window, width: i32, height: i32) {
    // make sure the viewport matches the new window dimensions; note that width and
    // height will be significantly larger than specified on retina displays.
    unsafe {
        gl::Viewport(0, 0, width, height);
    }
}

fn mouse_handler(state: &mut State, xposIn: f64, yposIn: f64) {
    let xpos = xposIn as f32;
    let ypos = yposIn as f32;

    if state.firstMouse {
        state.lastX = xpos;
        state.lastY = ypos;
        state.firstMouse = false;
    }

    let xoffset = xpos - state.lastX;
    let yoffset = state.lastY - ypos; // reversed since y-coordinates go from bottom to top

    state.lastX = xpos;
    state.lastY = ypos;

    state.camera.process_mouse_movement(xoffset, yoffset, true);
}

fn scroll_handler(state: &mut State, _xoffset: f64, yoffset: f64) {
    state.camera.process_mouse_scroll(yoffset as f32);
}

// utility function for loading a 2D texture from file
// ---------------------------------------------------
fn loadTexture(path: &str) -> GLuint {
    let mut texture_id: GLuint = 0;

    let img = image::open(path).expect("Texture failed to load");
    let (width, height) = (img.width() as GLsizei, img.height() as GLsizei);

    let color_type = img.color();
    // let data = img.into_rgb8().into_raw();

    unsafe {
        let format = match color_type {
            ColorType::L8 => gl::RED,
            // ColorType::La8 => {}
            ColorType::Rgb8 => gl::RGB,
            ColorType::Rgba8 => gl::RGBA,
            // ColorType::L16 => {}
            // ColorType::La16 => {}
            // ColorType::Rgb16 => {}
            // ColorType::Rgba16 => {}
            // ColorType::Rgb32F => {}
            // ColorType::Rgba32F => {}
            _ => panic!("no mapping for color type"),
        };

        let data = match color_type {
            ColorType::L8 => img.into_rgb8().into_raw(),
            // ColorType::La8 => {}
            ColorType::Rgb8 => img.into_rgb8().into_raw(),
            ColorType::Rgba8 => img.into_rgba8().into_raw(),
            // ColorType::L16 => {}
            // ColorType::La16 => {}
            // ColorType::Rgb16 => {}
            // ColorType::Rgba16 => {}
            // ColorType::Rgb32F => {}
            // ColorType::Rgba32F => {}
            _ => panic!("no mapping for color type"),
        };

        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            format as GLint,
            width,
            height,
            0,
            format,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const GLvoid,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
    }

    texture_id
}
