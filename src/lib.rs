use std::{ffi::CString, fmt::Display};

use crate::preprocessor::get_protocol_and_path;

pub mod shader;
pub mod program;
pub mod preprocessor;

fn create_whitespace_cstring(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}

#[derive(Debug, Clone)]
pub struct Path {
    protocol: Option<String>,
    components: Vec<String>,
}

impl Path {
    pub fn new(from: &str) -> Self {
        let (protocol, path) = get_protocol_and_path(from);
        let components = path.split(|c| c == '\\' || c == '/')
            .filter(|component| component.len() > 0 && component != &".");
    
        let mut final_components = vec![];
    
        for component in components {
            if component == ".." {
                let _ = final_components.pop();
            } else {
                final_components.push(component.to_string());
            }
        }
    
        Path { 
            protocol: protocol.map(|str| str.to_owned()), 
            components: final_components 
        }
    }

    pub fn join(&self, path: impl Into<Path>) -> Path {
        let path: Path = path.into();
        assert!(path.protocol.is_none());

        let mut result = self.clone();
        result.components.extend(path.components);
        result
    }

    pub fn pop(&mut self) -> Option<String> {
        self.components.pop()
    }

    pub fn dirname(&self) -> Path {
        let mut result = self.clone();
        result.pop();
        result
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.protocol {
            None => write!(f, "{}", self.components.join("/")),
            Some(protocol) => write!(f, "{protocol}://{}", self.components.join("/"))
        }
    }
}

impl Into<Path> for &str {
    fn into(self) -> Path {
        Path::new(self)
    }
}

impl Into<Path> for String {
    fn into(self) -> Path {
        Path::new(&self)
    }
}