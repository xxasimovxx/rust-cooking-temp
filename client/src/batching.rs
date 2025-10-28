#![allow(unused, dead_code)]
use crate::collision::Box3D;
use crate::helper::{self, create_texture};
use gl33::*;
use std::any::type_name;
use std::collections::HashMap;
use std::ffi::c_void;
use std::rc::Rc;
use tobj;
use ultraviolet::{Mat4, Vec3, Vec4};

const VERTEX_LEN: usize = size_of::<[f32; 8]>();

#[derive(Debug, Clone)]
pub struct BatchObject {
    pub vertex_data: Vec<f32>,
    pub indices: Vec<u32>,
    pub vertex_len: usize,
    pub position: Vec3,
    pub hitbox: Box3D,
}

impl BatchObject {
    pub fn new(gl: Rc<GlFns>, obj_path: &str, position: Vec3) -> Self {
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
        let mut vertex_data: Vec<f32> = vec![];
        let mut min_vertex: Vec3 = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max_vertex: Vec3 = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for i in 0..(positions.len() / 3) {
            let transform = Mat4::from_translation(position);

            let vert = Vec3::from(
                transform
                    * Vec4::new(
                        positions[3 * i],
                        positions[3 * i + 1],
                        positions[3 * i + 2],
                        1.0,
                    ),
            );

            min_vertex.x = min_vertex.x.min(vert.x);
            min_vertex.y = min_vertex.y.min(vert.y);
            min_vertex.z = min_vertex.z.min(vert.z);

            max_vertex.x = max_vertex.x.max(vert.x);
            max_vertex.y = max_vertex.y.max(vert.y);
            max_vertex.z = max_vertex.z.max(vert.z);

            let norm = Vec3::from(
                transform.inversed().transposed()
                    * Vec4::new(normals[3 * i], normals[3 * i + 1], normals[3 * i + 2], 1.0),
            );

            vertex_data.push(vert.x);
            vertex_data.push(vert.y);
            vertex_data.push(vert.z);

            vertex_data.push(norm.x);
            vertex_data.push(norm.y);
            vertex_data.push(norm.z);

            vertex_data.push(texcoords[2 * i]);
            vertex_data.push(texcoords[2 * i + 1]);
        }
        return Self {
            vertex_data: vertex_data,
            indices: indices,
            vertex_len: 8,
            position: position,
            hitbox: Box3D::new(max_vertex, min_vertex),
        };
    }
}

pub struct StaticBatch {
    vertex_data: Option<Vec<f32>>,
    indices: Option<Vec<u32>>,
    offset: usize,
    ebo: Option<helper::Buffer>,
    vbo: Option<helper::Buffer>,
    vao: Option<helper::VertexArray>,
}

// no hitboxes
impl StaticBatch {
    pub fn new() -> Self {
        Self {
            vertex_data: None,
            indices: None,
            offset: 0,
            vbo: None,
            ebo: None,
            vao: None,
        }
    }

    pub fn consume_object(&mut self, mut object: BatchObject) {
        match (&mut self.vertex_data, &mut self.indices) {
            (Some(vertex_data), Some(indices_data)) => {
                let offset = self.offset as u32;
                let mut new_indices_data = object.indices;
                new_indices_data.iter_mut().for_each(|elem| *elem += offset);
                self.offset += object.vertex_data.len() / object.vertex_len;

                vertex_data.append(&mut object.vertex_data);
                indices_data.append(&mut new_indices_data);
            }

            _ => {
                self.offset = object.vertex_data.len() / object.vertex_len;
                self.vertex_data = Some(object.vertex_data);
                self.indices = Some(object.indices);
            }
        }
    }

    pub fn send_data(&mut self, gl: Rc<GlFns>, texture_png_path: &str) {
        if self.vertex_data.is_none() || self.indices.is_none() {
            panic!("Data is empty!");
        }
        let vao = helper::VertexArray::new(gl.clone()).expect("Couldn`t make a VAO");
        vao.bind();

        let ebo = helper::Buffer::new(gl.clone()).expect("Couldn't make a EBO");
        ebo.bind(GL_ELEMENT_ARRAY_BUFFER);

        helper::buffer_data(
            gl.clone(),
            GL_ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&self.indices.clone().unwrap()),
            GL_STATIC_DRAW,
        );

        let vbo = helper::Buffer::new(gl.clone()).expect("Couldn't make a VBO");
        vbo.bind(GL_ARRAY_BUFFER);

        helper::buffer_data(
            gl.clone(),
            GL_ARRAY_BUFFER,
            bytemuck::cast_slice(&self.vertex_data.clone().unwrap()),
            GL_STATIC_DRAW,
        );

        unsafe {
            gl.VertexAttribPointer(
                0,
                3,
                GL_FLOAT,
                0,
                VERTEX_LEN.try_into().unwrap(),
                0 as *const _,
            );
            gl.EnableVertexAttribArray(0);

            gl.VertexAttribPointer(
                1,
                3,
                GL_FLOAT,
                0,
                VERTEX_LEN.try_into().unwrap(),
                size_of::<[f32; 3]>() as *const _,
            );
            gl.EnableVertexAttribArray(1);

            gl.VertexAttribPointer(
                2,
                2,
                GL_FLOAT,
                0,
                VERTEX_LEN.try_into().unwrap(),
                size_of::<[f32; 6]>() as *const _,
            );
            gl.EnableVertexAttribArray(2);
        }
        self.vao = Some(vao);
        self.vbo = Some(vbo);
        self.ebo = Some(ebo);
        create_texture(gl, texture_png_path);
    }

    pub fn draw(&self, gl: Rc<GlFns>) {
        match (&self.indices, &self.ebo, &self.vao, &self.vbo) {
            (Some(indices), Some(ebo), Some(vao), Some(vbo)) => unsafe {
                vao.bind();
                ebo.bind(GL_ELEMENT_ARRAY_BUFFER);
                vbo.bind(GL_ARRAY_BUFFER);
                gl.DrawElements(
                    GL_TRIANGLES,
                    indices.len().try_into().unwrap(), // rapair it later on
                    GL_UNSIGNED_INT,
                    0 as *const _,
                );
            },

            _ => {
                panic!("Shit happened!");
            }
        }
    }
}

/*
    DynamicBatch:
    0. same bahaviour as StaticBatch
    1. replace with other BatchObject of the same length and vertex size
    2. keep Offsets in hashmap
*/

// Shitfuck struct
pub struct Offset {
    offset: usize,
    vertex_data: Vec<f32>,
    position: Vec3,
    hitbox: Box3D,
}

pub struct DynamicBatch {
    vertex_data: Option<Vec<f32>>,
    indices: Option<Vec<u32>>,
    offset: usize,
    offset_map: HashMap<String, Offset>,
    ebo: Option<helper::Buffer>,
    vbo: Option<helper::Buffer>,
    vao: Option<helper::VertexArray>,
}

//offset is offset of Vertex elements( if vertex is vertices and normals an uv then one element is
//8 f32 long)
impl DynamicBatch {
    pub fn new() -> Self {
        Self {
            vertex_data: None,
            indices: None,
            offset: 0, // Its just offset needed to add new BatchObject
            offset_map: HashMap::new(),
            vbo: None,
            ebo: None,
            vao: None,
        }
    }

    pub fn consume_object(&mut self, mut object: BatchObject, name: &str) {
        match self.offset_map.get(name) {
            Some(_value) => {
                panic!("Name already taken in offset_map!");
            }
            None => {
                self.offset_map.insert(
                    name.to_string(),
                    Offset {
                        offset: self.offset * VERTEX_LEN,
                        vertex_data: object.vertex_data.clone(),
                        position: object.position,
                        hitbox: object.hitbox,
                    },
                );
            }
        }

        match (&mut self.vertex_data, &mut self.indices) {
            (Some(vertex_data), Some(indices_data)) => {
                let offset = self.offset as u32;
                let mut new_indices_data = object.indices;
                new_indices_data.iter_mut().for_each(|elem| *elem += offset);
                self.offset += object.vertex_data.len() / object.vertex_len;

                vertex_data.append(&mut object.vertex_data);
                indices_data.append(&mut new_indices_data);
            }

            _ => {
                self.offset = object.vertex_data.len() / object.vertex_len;
                self.vertex_data = Some(object.vertex_data);
                self.indices = Some(object.indices);
            }
        }
    }

    pub fn send_data(&mut self, gl: Rc<GlFns>, texture_png_path: &str) {
        if self.vertex_data.is_none() || self.indices.is_none() {
            panic!("Data is empty!");
        }
        let vao = helper::VertexArray::new(gl.clone()).expect("Couldn`t make a VAO");
        vao.bind();

        let ebo = helper::Buffer::new(gl.clone()).expect("Couldn't make a EBO");
        ebo.bind(GL_ELEMENT_ARRAY_BUFFER);

        helper::buffer_data(
            gl.clone(),
            GL_ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&self.indices.clone().unwrap()),
            GL_STATIC_DRAW,
        );

        let vbo = helper::Buffer::new(gl.clone()).expect("Couldn't make a VBO");
        vbo.bind(GL_ARRAY_BUFFER);

        helper::buffer_data(
            gl.clone(),
            GL_ARRAY_BUFFER,
            bytemuck::cast_slice(&self.vertex_data.clone().unwrap()),
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
        self.vao = Some(vao);
        self.vbo = Some(vbo);
        self.ebo = Some(ebo);
        create_texture(gl, texture_png_path);
    }

    pub fn draw(&self, gl: Rc<GlFns>) {
        match (&self.indices, &self.ebo, &self.vao, &self.vbo) {
            (Some(indices), Some(ebo), Some(vao), Some(vbo)) => unsafe {
                vao.bind();
                ebo.bind(GL_ELEMENT_ARRAY_BUFFER);
                vbo.bind(GL_ARRAY_BUFFER);
                gl.DrawElements(
                    GL_TRIANGLES,
                    indices.len().try_into().unwrap(), // rapair it later on
                    GL_UNSIGNED_INT,
                    0 as *const _,
                );
            },

            _ => {
                panic!("Shit happened!");
            }
        }
    }

    //
    //????????????????????????????????????????????????????????????????????????????
    //
    //
    //
    //
    //
    pub fn move_delta(&mut self, gl: Rc<GlFns>, name: &str, transformation: Mat4) {
        match self.offset_map.get_mut(name) {
            Some(elem) => {
                elem.position = Vec3::from(
                    transformation
                        * Vec4 {
                            x: elem.position.x,
                            y: elem.position.y,
                            z: elem.position.z,
                            w: 1.0
                        },
                );
                elem.hitbox.transformation(transformation);
                for i in 0..(elem.vertex_data.len() / 8) {
                    let i = i * 8;
                    let pos = transformation
                        * Vec4::new(
                            elem.vertex_data[i],
                            elem.vertex_data[i + 1],
                            elem.vertex_data[i + 2],
                            1.0,
                        );
                    let mut norm = transformation.inversed().transposed()
                        * Vec4::new(
                            elem.vertex_data[i + 3],
                            elem.vertex_data[i + 4],
                            elem.vertex_data[i + 5],
                            1.0,
                        );
                    norm.normalize();

                    elem.vertex_data[i] = pos.x;
                    elem.vertex_data[i + 1] = pos.y;
                    elem.vertex_data[i + 2] = pos.z;

                    elem.vertex_data[i + 3] = norm.x;
                    elem.vertex_data[i + 4] = norm.y;
                    elem.vertex_data[i + 5] = norm.z;
                }
                unsafe {
                    gl.BufferSubData(
                        GL_ARRAY_BUFFER,
                        (elem.offset) as isize,
                        (elem.vertex_data.len() * size_of::<f32>()) as isize,
                        elem.vertex_data.as_ptr() as *const c_void,
                    );
                }
            }
            None => {
                panic!("Shitfuck no name in map, function: move_delta!");
            }
        }
    }

    pub fn change_position(&mut self, gl: Rc<GlFns>, name: &str, position: Vec3) {
        match self.offset_map.get_mut(name) {
            Some(elem) => {
                let change_vec = position - elem.position;
                elem.position = position;

                elem.hitbox.move_delta(change_vec);

                for i in 0..(elem.vertex_data.len() / 8) {
                    let i = i * 8;
                    elem.vertex_data[i] += change_vec.x;
                    elem.vertex_data[i + 1] += change_vec.y;
                    elem.vertex_data[i + 2] += change_vec.z;
                    let mut norm = Mat4::from_translation(position).inversed().transposed()
                        * Vec4::new(
                            elem.vertex_data[i + 3],
                            elem.vertex_data[i + 4],
                            elem.vertex_data[i + 5],
                            1.0,
                        );
                    elem.vertex_data[i + 3] = norm.x;
                    elem.vertex_data[i + 4] = norm.y;
                    elem.vertex_data[i + 5] = norm.z;
                }
                unsafe {
                    gl.BufferSubData(
                        GL_ARRAY_BUFFER,
                        (elem.offset) as isize,
                        (elem.vertex_data.len() * size_of::<f32>()) as isize,
                        elem.vertex_data.as_ptr() as *const c_void,
                    );
                }
            }
            None => {
                panic!("Shitfuck no name in map, function: change_position");
            }
        }
    }

    pub fn hitbox_as_ref(&mut self, name: &str) -> Option<&Box3D>{
        match self.offset_map.get_mut(name) {
            Some(elem) => {
                return Some(&elem.hitbox);

            }
            None => {
                panic!("Fuck you!");
            }
        }
    }

    pub fn hitbox_as_mut(&mut self, name: &str) -> Option<&mut Box3D>{
        match self.offset_map.get_mut(name) {
            Some(elem) => {
                return Some(&mut elem.hitbox);

            }
            None => {
                panic!("Fuck you!");
            }
        }
    }
}
