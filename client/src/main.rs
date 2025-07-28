use beryllium::*;
use events::SDL_Keycode;
use gl33::*;
mod helper;
mod object;
use std::ffi::CString;
use std::fs;
use ultraviolet::*;

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
    unsafe {
        gl.fns.Enable(GL_DEPTH_TEST);
    }
    helper::clear_color(&gl.fns, 1.0, 1.0, 1.0, 1.0);
    let cube = object::Object::new(&gl.fns, "obj/brick.obj", "textures/red_brick.png");

    let vert_shader = fs::read_to_string("src/shader/vert.glsl").unwrap();
    let frag_shader = fs::read_to_string("src/shader/frag.glsl").unwrap();

    let shader_program =
        helper::ShaderProgram::from_vert_frag(&gl.fns, &vert_shader, &frag_shader).unwrap();
    shader_program.use_program(&gl.fns);
    let texture_image = CString::new("texture_image").unwrap();
    let texture_loc = unsafe {
        gl.fns
            .GetUniformLocation(shader_program.0, texture_image.as_ptr().cast())
    };

    unsafe {
        gl.fns.Uniform1i(texture_loc, 0);
    };

    let model_loc = unsafe {
        let name = CString::new("model").unwrap();
        gl.fns
            .GetUniformLocation(shader_program.0, name.as_ptr().cast())
    };
    let view_loc = unsafe {
        let name = CString::new("view").unwrap();
        gl.fns
            .GetUniformLocation(shader_program.0, name.as_ptr().cast())
    };
    let projection_loc = unsafe {
        let name = CString::new("projection").unwrap();
        gl.fns
            .GetUniformLocation(shader_program.0, name.as_ptr().cast())
    };
    println!(
        "{} {} {} {} ",
        projection_loc, view_loc, model_loc, texture_loc
    );

    let view = Mat4::from_translation(Vec3::new(0.0, 0.0, -2.0));
    unsafe { gl.fns.UniformMatrix4fv(view_loc, 1, 0, view.as_ptr()) };

    let projection = ultraviolet::projection::perspective_gl(
        45.0_f32.to_radians(),
        (800 as f32) / (600 as f32),
        0.1,
        100.0,
    );
    unsafe {
        gl.fns
            .UniformMatrix4fv(projection_loc, 1, 0, projection.as_ptr())
    };

    'main_loop: loop {
        while let Some((event, _timestamp)) = sdl.poll_events() {
            match event {
                events::Event::Quit => break 'main_loop,
                _ => (),
            }
        }
        let time = sdl.get_ticks() as f32 / 1000.0_f32;
        let mut model =
            Mat4::from_rotation_y(1.0) * Mat4::from_rotation_x(0.5) * Mat4::from_rotation_z(time);
        model.translate(&Vec3::new(0.0, 0.0, -5.0));
        helper::clear_color(&gl.fns, 0.8, 0.6, 1.0, 1.0);

        unsafe {
            gl.fns.Clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            gl.fns.UniformMatrix4fv(model_loc, 1, 0, model.as_ptr());
        }

        cube.draw(&gl.fns);
        //        helper::print_error(&gl.fns);
        gl.win.swap_window();
    }

    shader_program.delete(&gl.fns);
}
