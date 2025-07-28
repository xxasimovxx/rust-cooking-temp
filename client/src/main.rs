use beryllium::*;
use gl33::*;
mod helper;
mod object;
use std::fs;
use ultraviolet::*;

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

    let cube = object::Object::new(&gl.fns, "obj/brick.obj", "textures/red_brick.png");

    let vert_shader = fs::read_to_string("src/shader/vert.glsl").unwrap();
    let frag_shader = fs::read_to_string("src/shader/frag.glsl").unwrap();

    let shader_program =
        helper::ShaderProgram::from_vert_frag(&gl.fns, &vert_shader, &frag_shader).unwrap();
    shader_program.use_program(&gl.fns);

    let texture_loc = gl.get_uniform_location(&shader_program, "texture_image");

    let model_loc = gl.get_uniform_location(&shader_program, "model");

    let view_loc = gl.get_uniform_location(&shader_program, "view");

    let projection_loc = gl.get_uniform_location(&shader_program, "projection");

    println!(
        "projection_loc: {} view_loc: {} model_loc: {} texture_loc:{} ",
        projection_loc, view_loc, model_loc, texture_loc
    );

    let view = Mat4::from_translation(Vec3::new(0.0, 0.0, -5.0));
    gl.uniform_mat4fv(view_loc, view);

    let projection = ultraviolet::projection::perspective_gl(
        45.0_f32.to_radians(),
        (800 as f32) / (600 as f32),
        0.1,
        100.0,
    );
    gl.uniform_mat4fv(projection_loc, projection);

    'main_loop: loop {
        while let Some((event, _timestamp)) = sdl.poll_events() {
            match event {
                events::Event::Quit => break 'main_loop,
                _ => (),
            }
        }
        let time = sdl.get_ticks() as f32 / 1000.0_f32;
        let model =
            Mat4::from_rotation_y(1.0) * Mat4::from_rotation_x(0.5) * Mat4::from_rotation_z(time);

        gl.uniform_mat4fv(model_loc, model);

        gl.clear_color(0.8, 0.6, 1.0, 1.0);

        gl.clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        cube.draw(&gl.fns);

        //        helper::print_error(&gl.fns);
        gl.win.swap_window();
    }

    shader_program.delete(&gl.fns);
}
