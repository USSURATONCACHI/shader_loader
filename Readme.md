## shader_loader

`shader_loader` provides an easy-to-use API for handling OpenGL shaders, supporting multiple shader stages and custom file protocols. It also extends shader functionality with an `#include_once` directive, allowing you to modularize and reuse shader code just like in traditional C/C++ preprocessor environments.

### Features

- **Automatic Shader Compilation:** Load shaders from files or strings and compile them with ease.
- **Shader Program Management:** Create and manage shader programs with built-in error handling.
- **Custom Preprocessor Directives:** Use `#include_once` to include files and avoid code duplication.
- **Custom File Protocols:** Define your own file protocols for loading shader files from various sources.

### Getting Started

Here's how you can start using `shader_loader` in your Rust project:

#### 1. Add `shader_loader` to Your `Cargo.toml`

```toml
[dependencies]
shader_loader = { git = "https://github.com/USSURATONCACHI/shader_loader" }
```

#### 2. Load and Compile Shaders

You can create a shader from a file or a source string. Here's how to do both:

From a File:
```rust
use shader_loader::shader::Shader;
use std::path::PathBuf;

let vertex_shader = Shader::from_file(PathBuf::from("shader.vert"), gl::VERTEX_SHADER)
    .expect("Failed to load vertex shader");
```

From a Source String:
```rust
use shader_loader::shader::Shader;

let source = "
    #version 330 core
    layout(location = 0) in vec3 aPos;
    void main() {
        gl_Position = vec4(aPos, 1.0);
    }
";

let vertex_shader = Shader::from_source_str(source, gl::VERTEX_SHADER)
    .expect("Failed to compile vertex shader");
```

#### 3. Create a Shader Program

Combine shaders into a program and use it:
```rust
use shader_loader::program::Program;

// This function will collect all files based on provided path.
// path "shader" will expand into "shader.vert", "shader.frag", "shader.geom", "shader.comp"
// fir vertex, fragment, geometry, and compute shaders.
let program = Program::from_files_auto("shader")
    .expect("Failed to create shader program");

program.use_program(); // Analogous to glUseProgram
```

#### 4. Handle Shader Errors

If there's an error in your shader code, shader_loader provides detailed error messages with file and line information:

```rust
let result = Shader::from_file(PathBuf::from("shader.vert"), gl::VERTEX_SHADER);
match result {
    Ok(_) => println!("Shader compiled successfully"),
    Err(e) => println!("Shader compilation failed: {}", e),
}
```

#### 5. Using Custom File Protocols

You can add custom protocols to load shader files from different sources:
``` rust
use shader_loader::preprocessor::FileLoader;

fn custom_loader(path: &str) -> Result<String, String> {
    // Custom logic to load files
    Ok("Custom shader code".to_owned())
}

let mut loader = FileLoader::new();
loader.add_protocol("custom".to_owned(), custom_loader).unwrap();

let file = loader.load_file("custom://path/to/shader").unwrap();
```

And you can use `#include_once` directive to reuse shader code, if you are using `FileLoader`:
```glsl
#version 430 core

#include_once common.glsl
#include_once mat5.glsl
#include_once objects.glsl

layout (location = 0) in vec3 v_pos;

void calculate_objects_ray() {
    // ...
}
```

```rust
fn load_program(vert: &str, frag: &str) -> Program {
    // FileLoader::new() will handle local files by default,
    // but custom protocols could be implemented
    match Program::from_loader(&FileLoader::new(), &[
        (frag, gl::FRAGMENT_SHADER),
        (vert, gl::VERTEX_SHADER)
    ]) {
        Ok(program) => program,
        Err(error) => {
            panic!("\n{}", error);
        }
    }
}
```

### If README is inconsistent with actual code - add an Issue.
