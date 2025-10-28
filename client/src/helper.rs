#![allow(unused)]
use beryllium::{events::SDL_Keycode, *};
use gl33::*;
use imagine;
use std::collections::HashSet;
use std::ffi::CString;
use std::rc::Rc;
use ultraviolet::{Mat4, Vec3};
use video::GlWindow;

use crate::collision::Box3D;

pub struct GlFnsWin {
    pub fns: Rc<GlFns>,
    pub win: GlWindow,
}

impl GlFnsWin {
    pub fn new(sdl: &Sdl) -> Self {
        let win_args = video::CreateWinArgs {
            title: "window",
            width: 1024,
            height: 920,
            allow_high_dpi: true,
            borderless: false,
            resizable: false,
        };

        let win = sdl
            .create_gl_window(win_args)
            .expect("couldn't make a window and context");

        let gl = unsafe {
            GlFns::load_from(&|c_char_ptr| win.get_proc_address(c_char_ptr.cast())).unwrap()
        };

        return Self {
            fns: Rc::new(gl),
            win: win,
        };
    }

    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        clear_color(&self.fns, r, g, b, a);
    }

    pub fn enable(&self, value: GLenum) {
        enable_option(&self.fns, value);
    }

    pub fn get_uniform_location(&self, shader_program: &ShaderProgram, name: &str) -> i32 {
        get_uniform_location(&self.fns, shader_program, name)
    }
    pub fn uniform_mat4fv(&self, uniform_location: i32, mat4: Mat4) {
        uniform_mat4fv(&self.fns, uniform_location, mat4);
    }

    pub fn clear(&self, mask: GLbitfield) {
        clear_gl_bitfield(&self.fns, mask);
    }

    pub fn print_error(&self){

        print_error(self.fns.clone());
    }
}
#[derive(Clone)]
pub struct VertexArray(pub u32, Rc<GlFns>);

impl VertexArray {
    pub fn new(gl: Rc<GlFns>) -> Option<Self> {
        let mut vao = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut vao);
        }

        if vao != 0 {
            return Some(Self(vao, gl.clone()));
        } else {
            return None;
        }
    }

    pub fn bind(&self) {
        self.1.BindVertexArray(self.0)
    }

    pub fn _clear_binding(&self) {
        self.1.BindVertexArray(0);
    }

    pub fn delete(&self) {
        unsafe {
            self.1.DeleteVertexArrays(1, &self.0);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        self.delete();
    }
}
#[derive(Clone)]
pub struct Buffer(pub u32, Rc<GlFns>);

impl Buffer {
    pub fn new(gl: Rc<GlFns>) -> Option<Self> {
        let mut buffer = 0;
        unsafe {
            gl.GenBuffers(1, &mut buffer);
        }
        if buffer != 0 {
            Some(Self(buffer, gl.clone()))
        } else {
            None
        }
    }

    pub fn bind(&self, ty: GLenum) {
        unsafe { self.1.BindBuffer(ty, self.0) }
    }

    pub fn clear_binding(&self, ty: GLenum) {
        unsafe { self.1.BindBuffer(ty, 0) }
    }

    pub fn delete(&self) {
        unsafe {
            self.1.DeleteBuffers(1, &self.0);
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.delete();
    }
}

pub fn buffer_data(gl: Rc<GlFns>, ty: GLenum, data: &[u8], usage: GLenum) {
    unsafe {
        gl.BufferData(
            ty,
            data.len().try_into().unwrap(),
            data.as_ptr().cast(),
            usage,
        );
    }
}

#[derive(Clone)]
pub struct Shader(pub u32, Rc<GlFns>);
impl Shader {
    pub fn new(gl: Rc<GlFns>, ty: GLenum) -> Option<Self> {
        let shader = gl.CreateShader(ty);
        if shader != 0 {
            Some(Self(shader, gl.clone()))
        } else {
            None
        }
    }

    pub fn set_source(&self, src: &str) {
        unsafe {
            self.1.ShaderSource(
                self.0,
                1,
                &(src.as_bytes().as_ptr().cast()),
                &(src.len().try_into().unwrap()),
            );
        }
    }

    pub fn compile(&self) {
        self.1.CompileShader(self.0);
    }

    pub fn compile_success(&self) -> bool {
        let mut compiled = 0;
        unsafe { self.1.GetShaderiv(self.0, GL_COMPILE_STATUS, &mut compiled) };
        compiled == 1
    }

    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe {
            self.1
                .GetShaderiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len)
        };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            self.1.GetShaderInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    pub fn delete(&self) {
        self.1.DeleteShader(self.0);
    }

    pub fn from_source(gl: Rc<GlFns>, ty: GLenum, source: &str) -> Result<Self, String> {
        let id = Self::new(gl, ty).ok_or_else(|| "Couldn't allocate new shader".to_string())?;
        id.set_source(source);
        id.compile();
        if id.compile_success() {
            Ok(id)
        } else {
            let out = id.info_log();
            id.delete();
            Err(out)
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        self.delete();
    }
}

#[derive(Clone)]
pub struct ShaderProgram(pub u32, Rc<GlFns>);
impl ShaderProgram {
    pub fn new(gl: Rc<GlFns>) -> Option<Self> {
        let prog = gl.CreateProgram();
        if prog != 0 {
            Some(Self(prog, gl.clone()))
        } else {
            None
        }
    }

    pub fn attach_shader(&self, shader: &Shader) {
        self.1.AttachShader(self.0, shader.0);
    }

    pub fn link_program(&self) {
        self.1.LinkProgram(self.0);
    }

    pub fn link_success(&self) -> bool {
        let mut success = 0;
        unsafe { self.1.GetProgramiv(self.0, GL_LINK_STATUS, &mut success) };
        success == 1
    }

    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe {
            self.1
                .GetProgramiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len)
        };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            self.1.GetProgramInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    pub fn use_program(&self) {
        self.1.UseProgram(self.0);
    }

    pub fn delete(&self) {
        self.1.DeleteProgram(self.0);
    }

    pub fn from_vert_frag(gl: Rc<GlFns>, vert: &str, frag: &str) -> Result<Self, String> {
        let p = Self::new(gl.clone()).ok_or_else(|| "Couldn't allocate a program".to_string())?;
        let v = Shader::from_source(gl.clone(), GL_VERTEX_SHADER, vert)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let f = Shader::from_source(gl.clone(), GL_FRAGMENT_SHADER, frag)
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;
        p.attach_shader(&v);
        p.attach_shader(&f);
        p.link_program();
        v.delete();
        f.delete();
        if p.link_success() {
            Ok(p)
        } else {
            let out = format!("Program Link Error: {}", p.info_log());
            p.delete();
            Err(out)
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        self.delete();
    }
}

#[inline]
pub fn vec3_uniform(
    gl: &GlFns,
    shader_program: &ShaderProgram,
    name: &str,
    (v1, v2, v3): (&f32, &f32, &f32),
) {
    let name = CString::new(name).unwrap();
    unsafe {
        let location = gl.GetUniformLocation(shader_program.0, &name.as_bytes()[0] as *const u8);
        gl.Uniform3f(location, *v1, *v2, *v3);
    }
}

#[inline]
pub fn create_bitmap_from_png(path: &str) -> imagine::Bitmap {
    let mut f = std::fs::File::open(path).unwrap();
    let mut bytes = vec![];
    std::io::Read::read_to_end(&mut f, &mut bytes).unwrap();
    let mut bitmap = imagine::png::png_try_bitmap_rgba(&bytes, true).unwrap();
    bitmap.vertical_flip();
    return bitmap;
}

//Hehe not dropped
pub fn create_texture(gl: Rc<GlFns>, path: &str) -> u32 {
    let bitmap = create_bitmap_from_png(path);
    let mut texture = 0;
    unsafe {
        gl.GenTextures(1, &mut texture);
        gl.ActiveTexture(GL_TEXTURE0);
        gl.BindTexture(GL_TEXTURE_2D, texture);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT.0 as i32);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT.0 as i32);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR.0 as i32);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR.0 as i32);
        gl.TexImage2D(
            GL_TEXTURE_2D,
            0,
            GL_RGBA.0 as i32,
            bitmap.width.try_into().unwrap(),
            bitmap.height.try_into().unwrap(),
            0,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            bitmap.pixels.as_ptr().cast(),
        );
        gl.GenerateMipmap(GL_TEXTURE_2D);
    }
    return texture;
}

#[inline]
pub fn print_error(gl: Rc<GlFns>) {
    unsafe {
        println!("{:?}", gl.GetError());
    }
}

#[inline]
pub fn clear_color(gl: &GlFns, r: f32, g: f32, b: f32, a: f32) {
    unsafe { gl.ClearColor(r, g, b, a) }
}

#[inline]
pub fn enable_option(gl: &GlFns, value: GLenum) {
    unsafe {
        gl.Enable(value);
    }
}

#[inline]
pub fn get_uniform_location(gl: &GlFns, shader_program: &ShaderProgram, name: &str) -> i32 {
    let uniform_name = CString::new(name).unwrap();
    unsafe { gl.GetUniformLocation(shader_program.0, uniform_name.as_ptr().cast()) }
}

#[inline]
pub fn uniform_mat4fv(gl: &GlFns, uniform_location: i32, mat4: Mat4) {
    unsafe {
        gl.UniformMatrix4fv(uniform_location, 1, 0, mat4.as_ptr());
    }
}

#[inline]
pub fn clear_gl_bitfield(gl: &GlFns, mask: GLbitfield) {
    unsafe {
        gl.Clear(mask);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EulerFPSCamera {
    pub position: Vec3,
    pitch_deg: f32,
    yaw_deg: f32,
    pub hitbox: Box3D,
}
impl EulerFPSCamera {
    const UP: Vec3 = Vec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    fn make_front(&self) -> Vec3 {
        let pitch_rad = f32::to_radians(self.pitch_deg);
        let yaw_rad = f32::to_radians(self.yaw_deg);
        Vec3 {
            x: yaw_rad.sin() * pitch_rad.cos(),
            y: pitch_rad.sin(),
            z: yaw_rad.cos() * pitch_rad.cos(),
        }
    }

    fn make_front_perpendicular(&self) -> Vec3 {
        let pitch_rad = f32::to_radians(self.pitch_deg);
        let yaw_rad = f32::to_radians(self.yaw_deg);
        Vec3 {
            x: yaw_rad.sin() * pitch_rad.cos(),
            y: 0.0,
            z: yaw_rad.cos() * pitch_rad.cos(),
        }
    }

    pub fn update_orientation(&mut self, d_pitch_deg: f32, d_yaw_deg: f32) {
        self.pitch_deg = (self.pitch_deg + d_pitch_deg).max(-89.0).min(89.0);
        self.yaw_deg = (self.yaw_deg + d_yaw_deg) % 360.0;
    }

    pub fn update_position(&mut self, keys: &HashSet<SDL_Keycode>, distance: f32) {
        let forward = self.make_front_perpendicular();

        // #[cfg(debug_assertions)]
        // let forward = self.make_front();

        let cross_normalized = forward.cross(Self::UP).normalized();

        let mut move_vector = keys.iter().copied().fold(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            |vec, key| match key.0 {
                119 => vec + forward,
                115 => vec - forward,
                97 => vec - cross_normalized,
                100 => vec + cross_normalized,
                _ => vec,
            },
        );
        if !(move_vector.x == 0.0 && move_vector.y == 0.0 && move_vector.z == 0.0) {
            move_vector = move_vector.normalized();
            self.hitbox.move_delta(move_vector*distance);
            self.position += move_vector * distance;
        }
    }

    #[inline]
    pub fn make_view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.position, self.position + self.make_front(), Self::UP)
    }

    #[inline]
    pub fn at_position(position: Vec3) -> Self {
        let hitbox = Box3D::new(
            position
                + Vec3 {
                    x: 2.1,
                    y: 100.01,
                    z: 2.1,
                },
            position
                - Vec3 {
                    x: 2.1,
                    y: 100.01,
                    z: 2.1,
                },
        );
        Self {
            position,
            pitch_deg: 0.0,
            yaw_deg: 0.0,
            hitbox: hitbox,
        }
    }
}
