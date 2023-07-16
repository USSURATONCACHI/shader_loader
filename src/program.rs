use std::path::PathBuf;

use crate::{create_whitespace_cstring, shader::Shader};


pub trait Uniformable {
    unsafe fn set_uniform(self, location: i32);
}


pub struct Program(gl::types::GLuint);

impl Program {
    #[allow(dead_code)]
    pub fn from_files_auto(shader_name: &str) -> Result<Program, String> {
        const POSSIBLE_EXTS: [(&str, gl::types::GLenum); 4] = [
            (".vert", gl::VERTEX_SHADER),
            (".geom", gl::GEOMETRY_SHADER),
            (".frag", gl::FRAGMENT_SHADER),
            (".comp", gl::COMPUTE_SHADER),
        ];

        let files: Box<[_]> = POSSIBLE_EXTS.iter()
            .map(|(ext, shader_type)| (
                format!("{shader_name}{ext}"),
                shader_type.clone()
            ))
            .filter(|(path, _)| PathBuf::from(path).is_file())
            .collect();

        let files_ref: Box<[_]> = files.iter()
            .map(|(path, stype)| (path.as_str(), stype.clone()))
            .collect();

        Self::from_files(&files_ref)
    }

    #[allow(dead_code)]
    pub fn from_files(files: &[(&str, gl::types::GLenum)]) -> Result<Program, String> {
        let shaders: Result<Box<[_]>, _> = files
            .iter()
            .map(
                |(path, shader_type)| 
                    Shader::from_file(path.into(), *shader_type)
                        .map_err(|err| format!("File {path} :: {err}"))    
            )
            .collect();

        let shaders = shaders?;
        Self::from_shaders(&shaders)
    }

    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
		let program_id = unsafe { gl::CreateProgram() };

		for s in shaders {
			unsafe { gl::AttachShader(program_id, s.id()) };
		}

		unsafe { gl::LinkProgram(program_id) };
		let mut success: gl::types::GLint = 1;
		unsafe {
		    gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
		}

		if success == 0 {
		    let mut len: gl::types::GLint = 0;
		    unsafe {
		        gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
		    }

		    let error = create_whitespace_cstring(len as usize);

		    unsafe {
		        gl::GetProgramInfoLog(
		            program_id,
		            len,
		            std::ptr::null_mut(),
		            error.as_ptr() as *mut gl::types::GLchar
		        );
		    }

		    return Err(error.to_string_lossy().into_owned());
		}

		for s in shaders {
			unsafe { gl::DetachShader(program_id, s.id()) };
		}

        unsafe { gl::UseProgram(program_id); }
        Ok(Program(program_id))
	}

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.0);
        }
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.0
    }

    pub fn uniform<T: Uniformable>(&self, name: &str, val: T) {
        self.use_program();
        let location = gl_get_uniform_location(self, name);
        unsafe { 
            val.set_uniform(location); 
        }
    }
    
    pub fn location(&self, name: &str) -> i32 {
        gl_get_uniform_location(self, name)
    } 
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.0);
        }
    }
}

macro_rules! uniformable {
    ($type:ty, $function_name:expr) => {
        impl Uniformable for $type {
            unsafe fn set_uniform(self, location: i32) {
                $function_name (location, self)
            }
        }
    };

    ($type:ty, $function_name:expr, 2) => {
        impl Uniformable for $type {
            unsafe fn set_uniform(self, location: i32) {
                $function_name (location, self.0, self.1)
            }
        }
    };
    
    ($type:ty, $function_name:expr, 3) => {
        impl Uniformable for $type {
            unsafe fn set_uniform(self, location: i32) {
                $function_name (location, self.0, self.1, self.2)
            }
        }
    };

    
    ($type:ty, $function_name:expr, 4) => {
        impl Uniformable for $type {
            unsafe fn set_uniform(self, location: i32) {
                $function_name (location, self.0, self.1, self.2, self.3)
            }
        }
    };
}

uniformable!(f32, gl::Uniform1f);
uniformable!((f32, f32), gl::Uniform2f, 2);
uniformable!((f32, f32, f32), gl::Uniform3f, 3);
uniformable!((f32, f32, f32, f32), gl::Uniform4f, 4);

uniformable!(u32, gl::Uniform1ui);
uniformable!((u32, u32), gl::Uniform2ui, 2);
uniformable!((u32, u32, u32), gl::Uniform3ui, 3);
uniformable!((u32, u32, u32, u32), gl::Uniform4ui, 4);

uniformable!(i32, gl::Uniform1i);
uniformable!((i32, i32), gl::Uniform2i, 2);
uniformable!((i32, i32, i32), gl::Uniform3i, 3);
uniformable!((i32, i32, i32, i32), gl::Uniform4i, 4);


pub fn gl_get_uniform_location(program: &Program, name: &str) -> i32 {
    unsafe {
        let c_str = std::ffi::CString::new(name).unwrap();
        gl::GetUniformLocation(program.id(), c_str.as_ptr())
    }
}