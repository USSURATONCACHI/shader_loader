use std::{path::PathBuf, ffi::{CString, CStr}};

use crate::create_whitespace_cstring;


pub struct Shader(gl::types::GLuint);

impl Shader {
    pub fn from_file(file: PathBuf, shader_type: gl::types::GLenum) -> Result<Self, String> {
        assert!(file.is_file());
        let string = std::fs::read_to_string(file)
            .map_err(|err| format!("{err}"))?;

        Self::from_source_str(&string, shader_type)
    }

    pub fn from_source_str(source: &str, shader_type: gl::types::GLenum) -> Result<Self, String> {
        let c_string = CString::new(source).unwrap();
        Self::from_source(&c_string, shader_type)
    }
    
    pub fn from_source_string(source: String, shader_type: gl::types::GLenum) -> Result<Self, String> {
        let c_string = CString::new(source).unwrap();
        Self::from_source(&c_string, shader_type)
    }

    pub fn from_source(source: &CStr, shader_type: gl::types::GLenum) -> Result<Self, String> {
        let id = unsafe { gl::CreateShader(shader_type) };

        //Проверка на успешную компиляцию
        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }

        if success == 0 {
            //Получение длины текста ошибки и самого текста
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error: CString = create_whitespace_cstring(len as usize);

            unsafe {
                gl::GetShaderInfoLog(id, len, std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar);
            }
            let error = error.to_string_lossy()
                .into_owned()
                .replace("\\n", "\nnnnn")
                .replace("\\0", "[END]");
            
            return Err(error);
        } 

        Ok(Shader(id))
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.0
    }
}


impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.0);
        }
    }
}