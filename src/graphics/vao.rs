use gl;
use gl::types as gl_t;

pub struct VertexArrayObject {
    id: gl_t::GLuint,
}

impl VertexArrayObject {
    pub fn new() -> VertexArrayObject {
        let mut id: gl_t::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        VertexArrayObject { id }
    }

    pub fn bind(vao: &Self) {
        unsafe {
            gl::BindVertexArray(vao.id);
        }
    }

    pub fn unbind_all() {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.id);
        }
    }
}
