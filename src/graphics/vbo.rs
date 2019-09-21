use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;

use gl;
use gl::types as gl_t;

pub struct VertexBufferObject {
    id: gl_t::GLuint,
}

impl VertexBufferObject {
    pub fn new<T>(size: usize, data: Option<&[T]>) -> VertexBufferObject {
        let mut id: gl_t::GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size * size_of::<T>()) as isize,
                if let Some(v) = data {
                    v.as_ptr() as *const c_void
                } else {
                    ptr::null()
                },
                gl::DYNAMIC_DRAW,
            );
        }
        VertexBufferObject { id }
    }

    pub fn bind(vbo: &Self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo.id);
        }
    }

    pub fn attrib_format(location: u32, elements: u32, stride: usize, offset: usize) {
        unsafe {
            gl::EnableVertexAttribArray(location);
            gl::VertexAttribPointer(
                location,
                elements as i32,
                gl::FLOAT,
                gl::FALSE,
                stride as i32,
                offset as *const c_void,
            );
        }
    }

    pub fn integer_attrib_format(location: u32, elements: u32, stride: usize, offset: usize) {
        unsafe {
            gl::EnableVertexAttribArray(location);
            gl::VertexAttribIPointer(
                location,
                elements as i32,
                gl::INT,
                stride as i32,
                offset as *const c_void,
            );
        }
    }

    pub fn write<T>(offset: usize, data: &[T]) {
        unsafe {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                (offset * size_of::<T>()) as isize,
                (data.len() * size_of::<T>()) as isize,
                data.as_ptr() as *const c_void,
            );
        }
    }

    pub fn unbind_all() {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

impl Drop for VertexBufferObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.id);
        }
    }
}
