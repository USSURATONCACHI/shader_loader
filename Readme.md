## shader_loader

This library automates some tedious work of loading and compiling OpenGL shaders. 

It also adds _optional_ quality-of-life feature, called `#include_once` - additional preprocessor directive that allows to include (in C preprocessor sense) once any file in any file format. This allows to move common logic of shades into single file and avoid code duplication.

By default OpenGL does not allow for `#include` directives in shaders.

## Quick start

