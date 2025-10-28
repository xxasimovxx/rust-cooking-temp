/*
        add print opengl error macro
*/
use beryllium::events::Event;
use beryllium::*;
use gl33::*;
mod batching;
mod collision;
mod helper;
mod object;
use std::collections::HashSet;
use std::fs;
use ultraviolet::*;

const OBJ_AMOUNT: usize = 4;

fn main() {
    let sdl = Sdl::init(init::InitFlags::EVERYTHING);
    sdl.set_gl_context_major_version(3).unwrap();
    sdl.set_gl_context_major_version(3).unwrap();
    sdl.set_relative_mouse_mode(true).unwrap();
    sdl.set_gl_profile(video::GlProfile::Core).unwrap();

    #[cfg(target_os = "macos")]
    {
        sdl.set_gl_context_flags(video::GlContextFlags::FORWARD_COMPATIBLE)
            .unwrap();
    }

    let gl = helper::GlFnsWin::new(&sdl);
    gl.enable(GL_DEPTH_TEST);

    let mut batcher = batching::DynamicBatch::new();
    for i in 0..OBJ_AMOUNT {
        for j in 0..OBJ_AMOUNT {
            let batch_obj_temp = batching::BatchObject::new(
                gl.fns.clone(),
                "obj/brick.obj",
                Vec3 {
                    x: 3.0 + 3.0 * i as f32,
                    y: 3.0 + 3.0 * j as f32,
                    z: 6.0,
                },
            );
            let mut name: String = "asd".to_owned();
            name.push_str(&i.to_string());
            name.push_str(" ");
            name.push_str(&j.to_string());
            batcher.consume_object(batch_obj_temp, &name);
        }
    }

    batcher.send_data(gl.fns.clone(), "textures/red_brick.png");

    let vert_shader = fs::read_to_string("src/shader/vert.glsl").unwrap();
    let frag_shader = fs::read_to_string("src/shader/frag.glsl").unwrap();

    let shader_program =
        helper::ShaderProgram::from_vert_frag(gl.fns.clone(), &vert_shader, &frag_shader).unwrap();
    shader_program.use_program();

    let texture_loc = gl.get_uniform_location(&shader_program, "texture_image");

    let model_loc = gl.get_uniform_location(&shader_program, "model");

    let view_loc = gl.get_uniform_location(&shader_program, "view");

    let projection_loc = gl.get_uniform_location(&shader_program, "projection");

    println!(
        "projection_loc: {} view_loc: {} model_loc: {} texture_loc:{} ",
        projection_loc, view_loc, model_loc, texture_loc
    );

    let view = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
    gl.uniform_mat4fv(view_loc, view);

    let projection = ultraviolet::projection::perspective_gl(
        45.0_f32.to_radians(),
        (800 as f32) / (600 as f32),
        0.1,
        100.0,
    );
    gl.uniform_mat4fv(projection_loc, projection);
    let mut camera = helper::EulerFPSCamera::at_position(Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
    let camera_speed = 100.0;
    sdl.set_relative_mouse_mode(true).unwrap();
    let mut keys_held = HashSet::new();
    let mut last_time = 0.0;
    let mut frame_count = 0.0;
    let mut previous_time = sdl.get_ticks() as f32 / 10_000.0_f32;

    'main_loop: loop {
        while let Some((event, _timestamp)) = sdl.poll_events() {
            match event {
                events::Event::Quit => break 'main_loop,
                Event::MouseMotion {
                    x_delta, y_delta, ..
                } => {
                    let d_yaw_deg = -x_delta as f32 * 0.1;
                    let d_pitch_deg = -y_delta as f32 * 0.1;
                    camera.update_orientation(d_pitch_deg, d_yaw_deg);
                }
                Event::Key {
                    pressed, keycode, ..
                } => {
                    if pressed {
                        keys_held.insert(keycode);
                    } else {
                        keys_held.remove(&keycode);
                    }
                }
                _ => (),
            }
        }
        frame_count += 1.0;
        let time = sdl.get_ticks() as f32 / 1000.0_f32;
        let delta_time = time / 10.0 - last_time;
        last_time = time / 10.0;

        if time - previous_time >= 1.0 {
            println!("{}", frame_count);
            previous_time = time;
            frame_count = 0.0;
        }

        camera.update_position(&keys_held, camera_speed * delta_time);

        let view: Mat4 = camera.make_view_matrix();

        gl.uniform_mat4fv(view_loc, view);

        let model = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0))
            * Mat4::from_rotation_y(0.0)
            * Mat4::from_rotation_x(0.0)
            * Mat4::from_rotation_z(time * 0.0);

        gl.uniform_mat4fv(model_loc, model);

        gl.clear_color(0.1, 0.1, 0.1, 1.0);

        gl.clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        batcher.draw(gl.fns.clone());
        let transformation = Mat4::from_translation(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });

        for i in 0..OBJ_AMOUNT {
            for j in 0..OBJ_AMOUNT {
                let mut name: String = "asd".to_owned();
                name.push_str(&i.to_string());
                name.push_str(" ");
                name.push_str(&j.to_string());
                batcher.hitbox_as_mut(&name).unwrap().transformation(model);
                if camera.hitbox.colide(batcher.hitbox_as_ref(&name).unwrap()){
                    println!("{}", name);

                }
                //batcher.change_position(gl.fns.clone(), &name, Vec3 { x: 5.0, y: 0.0, z: 0.0 });
                //batcher.move_delta(gl.fns.clone(), &name, transformation);
            }
        }

        //        helper::print_error(gl.fns.clone());
        gl.win.swap_window();
    }
}
