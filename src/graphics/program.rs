use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr;

use gl::types as gl_t;

// Creates a CString with the specified length
pub fn new_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}

// Converts a String into a CString
pub fn string_to_cstring(string: &str) -> CString {
    unsafe { CString::from_vec_unchecked(string.as_bytes().to_vec()) }
}

pub struct Program {
    id: gl_t::GLuint,
}

impl Program {
    pub fn make_current(program: &Self) {
        unsafe {
            gl::UseProgram(program.id);
        }
    }
    pub fn use_none() {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn get_uniform(&self, name: &str) -> Result<u32, String> {
        let uniform = unsafe { gl::GetUniformLocation(self.id, string_to_cstring(name).as_ptr()) };
        if uniform < 0 {
            Err(format!("'{}' is not a uniform", name))
        } else {
            Ok(uniform as u32)
        }
    }

    pub fn get_vertex_attrib(&self, name: &str) -> Result<u32, String> {
        let attrib = unsafe { gl::GetAttribLocation(self.id, string_to_cstring(name).as_ptr()) };
        if attrib < 0 {
            Err(format!("'{}' is not a vertex attribute", name))
        } else {
            Ok(attrib as u32)
        }
    }

    pub fn new(path: &Path) -> Result<Program, String> {
        unsafe {
            let program_id = gl::CreateProgram(); // Genereate the program ID

            let mut status: gl_t::GLint = 0;
            let mut string = String::new();

            let mut len: gl_t::GLint = 0;
            let mut log: CString;

            let vertex_shader;
            if let Ok(mut file) = File::open(path.with_file_name("vertex.glsl")) {
                // Open the file
                if let Ok(_) = file.read_to_string(&mut string) {
                    // Read the file
                    println!("Vertex shader source:\n{}\n", string);

                    // Create a new vertex shader
                    vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
                    // Attach the source code to it
                    gl::ShaderSource(
                        vertex_shader,
                        1,
                        &string_to_cstring(string.as_str()).as_ptr(),
                        ptr::null(),
                    );
                    gl::CompileShader(vertex_shader); // Compile it
                    gl::AttachShader(program_id, vertex_shader); // Attach it to the program

                    // Checking shader compile status
                    gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut status);
                    println!("Vertex shader compile status: {}", status);

                    if status == 0 {
                        // Get the legth of the info log
                        gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut len);
                        // Allocate the memory to store the log
                        log = new_cstring_with_len(len as usize);
                        // Retrive the info log
                        gl::GetShaderInfoLog(vertex_shader, len, &mut len, log.as_ptr() as *mut _);
                        println!(
                            "Vertex shader info log: {}\n",
                            log.into_string()
                                .or(Err("Can't convert the vertex shader info log to a String"))?
                        );
                    }
                } else {
                    return Err("Cannot read the vertex shader file!".to_string());
                }
            } else {
                return Err("Vertex shader not found!".to_string());
            }
            let fragment_shader;
            if let Ok(mut file) = File::open(path.with_file_name("fragment.glsl")) {
                // Open the file
                string.clear();
                if let Ok(_) = file.read_to_string(&mut string) {
                    // Read the file
                    println!("Fragment shader source:\n{}\n", string);

                    // Create a new fragment shader
                    fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
                    // Attach the source code to it
                    gl::ShaderSource(
                        fragment_shader,
                        1,
                        &string_to_cstring(string.as_str()).as_ptr(),
                        ptr::null(),
                    );
                    gl::CompileShader(fragment_shader); // Compile it
                    gl::AttachShader(program_id, fragment_shader); // Attach it to the program

                    // Checking shader compile status
                    gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut status);
                    println!("Fragment shader compile status: {}", status);

                    if status == 0 {
                        // Get the legth of the info log
                        gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut len);
                        // Allocate the memory to store the log
                        log = new_cstring_with_len(len as usize);
                        // Retrive the info log
                        gl::GetShaderInfoLog(
                            fragment_shader,
                            len,
                            &mut len,
                            log.as_ptr() as *mut _,
                        );
                        println!(
                            "Fragment shader info log: {}\n",
                            log.into_string()
                                .or(Err("Can't convert the fragment info log to a String"))?
                        );
                    }
                // else shaders += FRAGMENT_SHADER;
                } else {
                    return Err("Cannot read the fragment shader file!".to_string());
                }
            } else {
                return Err("Fragment shader not found!".to_string());
            }
            gl::LinkProgram(program_id);

            // Checking program link status
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut status);
            println!("Program link status: {}", status);

            if status == 0 {
                // Get the legth of the info log
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
                // Allocate the memory to store the log
                log = new_cstring_with_len(len as usize);
                // Retrive the info log
                gl::GetProgramInfoLog(program_id, len, &mut len, log.as_ptr() as *mut _);
                println!(
                    "Program info log: {}\n",
                    log.into_string()
                        .or(Err("Can't convert the program info log to a String"))?
                );
            }
            // Delete the shaders which are already linked
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            Ok(Program { id: program_id })
        }
    }
}
