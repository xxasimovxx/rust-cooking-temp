/*
*  implement Drop trait for helper module structs
*
*/
use beryllium::*;
use events::SDL_Keycode;
use gl33::*;
mod helper;
mod object;
use std::fs;

fn main() {
    type Vertex = [f32; 6];
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

    let vert_shader = fs::read_to_string("src/shader/vert.glsl").unwrap();
    let frag_shader = fs::read_to_string("src/shader/frag.glsl").unwrap();

    let shader_program =
        helper::ShaderProgram::from_vert_frag(&gl.fns, &vert_shader, &frag_shader).unwrap();
    shader_program.use_program(&gl.fns);

    let cube = object::Object::new(&gl.fns, "obj/cube.obj");

    'main_loop: loop {
        while let Some((event, _timestamp)) = sdl.poll_events() {
            match event {
                events::Event::Quit => break 'main_loop,
                _ => (),
            }
        }
        helper::clear_color(&gl.fns, 0.8, 0.6, 1.0, 1.0);

        unsafe {
            gl.fns.Clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        }

        cube.draw(&gl.fns);
        gl.win.swap_window();
    }

    shader_program.delete(&gl.fns);
}
