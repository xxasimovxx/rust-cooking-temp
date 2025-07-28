#![allow(unused)]
use beryllium::*;
use gl33::*;
use imagine;
use std::ffi::CString;
use video::GlWindow;

const BACKGROUND_COLOR: [f32; 3] = [0.7, 0.7, 0.5];
pub fn clear_color(gl: &GlFns, r: f32, g: f32, b: f32, a: f32) {
    unsafe { gl.ClearColor(r, g, b, a) }
}

pub struct GlFnsWin {
    pub fns: GlFns,
    pub win: GlWindow,
}

impl GlFnsWin {
    pub fn new(sdl: &Sdl) -> Self {
        let win_args = video::CreateWinArgs {
            title: "window",
            width: 800,
            height: 800,
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

        return Self { fns: gl, win: win };
    }
}
#[derive(Debug)]
pub struct VertexArray(pub u32);

impl VertexArray {
    pub fn new(gl: &GlFns) -> Option<Self> {
        let mut vao = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut vao);
        }

        if vao != 0 {
            return Some(Self(vao));
        } else {
            return None;
        }
    }

    pub fn bind(&self, gl: &GlFns) {
        gl.BindVertexArray(self.0)
    }

    pub fn _clear_binding(gl: GlFns) {
        gl.BindVertexArray(0)
    }

    pub fn delete(&self, gl: &GlFns) {
        unsafe {
            gl.DeleteVertexArrays(1, &self.0);
        }
    }
}
#[derive(Debug)]
pub struct Buffer(pub u32);

impl Buffer {
    pub fn new(gl: &GlFns) -> Option<Self> {
        let mut buffer = 0;
        unsafe {
            gl.GenBuffers(1, &mut buffer);
        }
        if buffer != 0 {
            Some(Self(buffer))
        } else {
            None
        }
    }

    pub fn bind(&self, gl: &GlFns, ty: GLenum) {
        unsafe { gl.BindBuffer(ty, self.0) }
    }

    pub fn clear_binding(gl: &GlFns, ty: GLenum) {
        unsafe { gl.BindBuffer(ty, 0) }
    }

    pub fn delete(&self, gl: &GlFns) {
        unsafe {
            gl.DeleteBuffers(1, &self.0);
        }
    }
}

pub fn buffer_data(gl: &GlFns, ty: GLenum, data: &[u8], usage: GLenum) {
    unsafe {
        gl.BufferData(
            ty,
            data.len().try_into().unwrap(),
            data.as_ptr().cast(),
            usage,
        );
    }
}

pub struct Shader(pub u32);
impl Shader {
    pub fn new(gl: &GlFns, ty: GLenum) -> Option<Self> {
        let shader = gl.CreateShader(ty);
        if shader != 0 {
            Some(Self(shader))
        } else {
            None
        }
    }

    pub fn set_source(&self, gl: &GlFns, src: &str) {
        unsafe {
            gl.ShaderSource(
                self.0,
                1,
                &(src.as_bytes().as_ptr().cast()),
                &(src.len().try_into().unwrap()),
            );
        }
    }

    pub fn compile(&self, gl: &GlFns) {
        gl.CompileShader(self.0);
    }

    pub fn compile_success(&self, gl: &GlFns) -> bool {
        let mut compiled = 0;
        unsafe { gl.GetShaderiv(self.0, GL_COMPILE_STATUS, &mut compiled) };
        compiled == 1
    }

    pub fn info_log(&self, gl: &GlFns) -> String {
        let mut needed_len = 0;
        unsafe { gl.GetShaderiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            gl.GetShaderInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    pub fn delete(self, gl: &GlFns) {
        gl.DeleteShader(self.0);
    }

    pub fn from_source(gl: &GlFns, ty: GLenum, source: &str) -> Result<Self, String> {
        let id = Self::new(gl, ty).ok_or_else(|| "Couldn't allocate new shader".to_string())?;
        id.set_source(gl, source);
        id.compile(gl);
        if id.compile_success(gl) {
            Ok(id)
        } else {
            let out = id.info_log(gl);
            id.delete(gl);
            Err(out)
        }
    }
}

pub struct ShaderProgram(pub u32);
impl ShaderProgram {
    pub fn new(gl: &GlFns) -> Option<Self> {
        let prog = gl.CreateProgram();
        if prog != 0 { Some(Self(prog)) } else { None }
    }

    pub fn attach_shader(&self, gl: &GlFns, shader: &Shader) {
        gl.AttachShader(self.0, shader.0);
    }

    pub fn link_program(&self, gl: &GlFns) {
        gl.LinkProgram(self.0);
    }

    pub fn link_success(&self, gl: &GlFns) -> bool {
        let mut success = 0;
        unsafe { gl.GetProgramiv(self.0, GL_LINK_STATUS, &mut success) };
        success == 1
    }

    pub fn info_log(&self, gl: &GlFns) -> String {
        let mut needed_len = 0;
        unsafe { gl.GetProgramiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            gl.GetProgramInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    pub fn use_program(&self, gl: &GlFns) {
        gl.UseProgram(self.0);
    }

    pub fn delete(self, gl: &GlFns) {
        gl.DeleteProgram(self.0);
    }

    pub fn from_vert_frag(gl: &GlFns, vert: &str, frag: &str) -> Result<Self, String> {
        let p = Self::new(gl).ok_or_else(|| "Couldn't allocate a program".to_string())?;
        let v = Shader::from_source(gl, GL_VERTEX_SHADER, vert)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let f = Shader::from_source(gl, GL_FRAGMENT_SHADER, frag)
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;
        p.attach_shader(gl, &v);
        p.attach_shader(gl, &f);
        p.link_program(gl);
        v.delete(gl);
        f.delete(gl);
        if p.link_success(gl) {
            Ok(p)
        } else {
            let out = format!("Program Link Error: {}", p.info_log(gl));
            p.delete(gl);
            Err(out)
        }
    }
}

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

pub fn create_bitmap_from_png(path: &str) -> imagine::Bitmap {
    let mut f = std::fs::File::open(path).unwrap();
    let mut bytes = vec![];
    std::io::Read::read_to_end(&mut f, &mut bytes).unwrap();
    let mut bitmap = imagine::png::png_try_bitmap_rgba(&bytes, true).unwrap();
    bitmap.vertical_flip();
    return bitmap;
}

pub fn create_texture(gl: &GlFns, path: &str) -> u32 {
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

pub fn print_error(gl: &GlFns) {
    unsafe {
        println!("{:?}", gl.GetError());
    }
}
