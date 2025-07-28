/*
* Possible performance issue creating new vao every object (will keep it that way to make thing
* elegant and change if performance issue met)
*/
use crate::helper::{self, create_texture};
use gl33::*;
use tobj;
#[allow(unused)]

pub struct Object {
    pub indices_len: i32,
    pub ebo: helper::Buffer,
    pub vbo: helper::Buffer,
    pub vao: helper::VertexArray,
}

impl Object {
    pub fn new(gl: &GlFns, obj_path: &str, texture_png_path: &str) -> Self {
        let (models, _) = tobj::load_obj(
            obj_path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .expect("Failed to load OBJ");

        let mesh = &models[0].mesh;

        let positions = &mesh.positions;
        let normals = &mesh.normals;
        let texcoords = &mesh.texcoords;
        let indices = mesh.indices.clone();
        println!(
            "positions: {} normals: {} texcoords: {} indices: {}",
            positions.len(),
            normals.len(),
            texcoords.len(),
            indices.len()
        );

        let mut vertex_data: Vec<f32> = vec![];
        for i in 0..(positions.len() / 3) {
            vertex_data.push(positions[3 * i]);
            vertex_data.push(positions[3 * i + 1]);
            vertex_data.push(positions[3 * i + 2]);

            vertex_data.push(normals[3 * i]);
            vertex_data.push(normals[3 * i + 1]);
            vertex_data.push(normals[3 * i + 2]);

            vertex_data.push(texcoords[2 * i]);
            vertex_data.push(texcoords[2 * i + 1]);
        }

        let vao = helper::VertexArray::new(gl).expect("Couldn`t make a VAO");
        vao.bind(gl);

        let ebo = helper::Buffer::new(&gl).expect("Couldn't make a EBO");
        ebo.bind(&gl, GL_ELEMENT_ARRAY_BUFFER);

        helper::buffer_data(
            &gl,
            GL_ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&indices),
            GL_STATIC_DRAW,
        );

        let vbo = helper::Buffer::new(&gl).expect("Couldn't make a VBO");
        vbo.bind(&gl, GL_ARRAY_BUFFER);

        helper::buffer_data(
            &gl,
            GL_ARRAY_BUFFER,
            bytemuck::cast_slice(&vertex_data),
            GL_STATIC_DRAW,
        );

        unsafe {
            gl.VertexAttribPointer(
                0,
                3,
                GL_FLOAT,
                0,
                size_of::<[f32; 8]>().try_into().unwrap(),
                0 as *const _,
            );
            gl.EnableVertexAttribArray(0);

            gl.VertexAttribPointer(
                1,
                3,
                GL_FLOAT,
                0,
                size_of::<[f32; 8]>().try_into().unwrap(),
                size_of::<[f32; 3]>() as *const _,
            );
            gl.EnableVertexAttribArray(1);

            gl.VertexAttribPointer(
                2,
                2,
                GL_FLOAT,
                0,
                size_of::<[f32; 8]>().try_into().unwrap(),
                size_of::<[f32; 6]>() as *const _,
            );
            gl.EnableVertexAttribArray(2);
        }
        create_texture(gl, texture_png_path);
        return Self {
            indices_len: indices.len().try_into().unwrap(),
            ebo: ebo,
            vbo: vbo,
            vao: vao,
        };
    }

    pub fn draw(&self, gl: &GlFns) {
        unsafe {
            self.vao.bind(gl);
            self.ebo.bind(gl, GL_ELEMENT_ARRAY_BUFFER);
            self.vbo.bind(gl, GL_ARRAY_BUFFER);
            gl.DrawElements(
                GL_TRIANGLES,
                self.indices_len,
                GL_UNSIGNED_INT,
                0 as *const _,
            );
        }
    }
}
