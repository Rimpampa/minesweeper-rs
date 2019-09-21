extern crate image;

use std::ffi::c_void;
use std::path::Path;
use std::ptr;

use gl;
use gl::types as gl_t;

pub struct Texture {
    id: gl_t::GLuint,
    width: usize,
    height: usize,
    pixel_size: (f32, f32),
}

impl Texture {
    pub fn new(width: usize, height: usize) -> Texture {
        let mut id: gl_t::GLuint = 0;
        unsafe {
            // Genereate a new texture
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            // Fill the texture with the pixel data from the image
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );
            // Set some texture parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }
        Texture {
            id,
            width,
            height,
            pixel_size: (1.0 / width as f32, 1.0 / height as f32),
        }
    }

    pub fn from_file(path: &Path) -> Result<Texture, String> {
        // Open the image buffer
        let img = image::open(path)
            .or_else(|e| Err(format!("{}", e)))?
            .to_rgba();

        let mut id: gl_t::GLuint = 0;
        unsafe {
            // Genereate a new texture
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            // Fill the texture with the pixel data from the image
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.to_vec().as_ptr() as *const c_void,
            );
            // Set some texture parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }
        Ok(Texture {
            id,
            width: img.width() as usize,
            height: img.height() as usize,
            pixel_size: (1.0 / img.width() as f32, 1.0 / img.height() as f32),
        })
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixel_size(&self) -> (f32, f32) {
        self.pixel_size
    }

    pub fn set_active_unit(unit: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + unit);
        }
    }

    pub fn unbind_all() {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}
