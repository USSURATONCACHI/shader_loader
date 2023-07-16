use std::{rc::Rc, collections::HashSet};

use regex::Regex;

/// Contains info about a segment of text being replaced by text from another file
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub start_line: usize,
    pub end_line: usize,
    pub original_file: Rc<String>,  // Just not to clone too many text
}

impl Segment {
    pub fn is_inside(&self, of: &Self) -> bool {
        self.start_line >= of.start_line && 
        self.end_line <= of.end_line
    }
}

/// [`FileIncludes`] is produced when preprocessor loads a file. It contains all the info
/// about text layout in the file. It is needed to be able to convert OpenGL errors of
/// final text blob into errors for each separate file.
/// 
/// If there is no need in layout info, you can just extract text via `text` method.
#[derive(Debug, Clone)]
pub struct FileIncludes {
    lines: Vec<String>,
    segments: Vec<Segment>, // Segments are required to be in order - child segments must lay AFTER parent segments
}

impl FileIncludes {
    pub fn new(text: &str, original_file: String) -> Self {
        let lines: Vec<_> = text.split("\n").into_iter().map(|s| s.to_owned()).collect();
        let end_line = lines.len();
        FileIncludes { 
            lines,
            segments: vec![Segment {
                start_line: 0,
                end_line,
                original_file: Rc::new(original_file)
            }]
        }
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    } 

    /// Returns the segment of this line
    pub fn last_segment_at(&self, line: usize) -> Option<Segment> {
        for segment in self.segments.iter().rev() {
            if line >= segment.start_line && line < segment.end_line {
                return Some(segment.clone());
            }
        }

        None
    }

    pub fn file_and_line_at(&self, line: usize) -> Option<(Rc<String>, usize)> {
        let segment = match self.last_segment_at(line) {
            None => return None,
            Some(s) => s,
        };

        let mut local_line = line - segment.start_line;

        for seg in self.segments.iter() {
            if self.get_segment_parent(seg.clone()) == Some(segment.clone()) {
                local_line -= seg.end_line - seg.start_line - 1;
            }
        }

        Some((segment.original_file, local_line))
    }

    pub fn get_segment_parent(&self, segment: Segment) -> Option<Segment> {
        let pos = match self.segments.iter().position(|s| s.eq(&segment)) {
            None => return None,
            Some(pos) => pos,
        };

        for potential_parent in self.segments[..pos].iter().rev() {
            if segment.is_inside(potential_parent) {
                return Some(potential_parent.clone());
            }
        }

        None
    }

    pub fn all_segments_at(&self, line: usize) -> Vec<Segment> {
        let mut vec = vec![];

        for segment in self.segments.iter() {
            if line >= segment.start_line && line < segment.end_line {
                vec.push(segment.clone());
            }
        }

        vec
    } 

    pub fn all_used_files(&self) -> Vec<&str> {
        let mut map = HashSet::new();

        for s in self.segments.iter() {
            map.insert(s.original_file.as_str());
        }

        map.into_iter().collect()
    }

    pub fn replace_line_with(&mut self, line: usize, with: &str, original_file: Rc<String>) {
        let insert_lines: Vec<_> = with.split("\n").map(|s| s.to_owned()).collect();
        let new_lines_count = insert_lines.len();
        
        self.lines.remove(line);
        for (i, new_line) in insert_lines.into_iter().enumerate() {
            self.lines.insert(i + line, new_line);
        }

        for segment in self.segments.iter_mut() {
            if segment.start_line > line {
                segment.start_line += new_lines_count;
                segment.start_line -= 1;
            }
            if segment.end_line > line {
                segment.end_line += new_lines_count;
                segment.end_line -= 1;
            }
        }

        self.segments.push(Segment { 
            start_line: line, 
            end_line: line + new_lines_count, 
            original_file, 
        });
    }

    pub fn replace_line_with_includes(&mut self, line: usize, includes: FileIncludes) {
        self.lines.remove(line); // Remove the line
        let new_lines_count = includes.lines.len();

        for (i, new_line) in includes.lines.into_iter().enumerate() {
            self.lines.insert(i + line, new_line);
        }

        for segment in self.segments.iter_mut() {
            if segment.start_line > line {
                segment.start_line += new_lines_count;
                segment.start_line -= 1;
            }
            if segment.end_line > line {
                segment.end_line += new_lines_count;
                segment.end_line -= 1;
            }
        }

        for mut new_segment in includes.segments.into_iter() {
            new_segment.start_line += line;
            new_segment.end_line += line;

            self.segments.push(new_segment);
        }
    }
}

pub type Protocol = dyn Fn(&str) -> Result<String, String>;

/// Loads files and unfolds `#include_once` preprocessor directives.
/// 
/// Also allows you to add your own protocols to load files from custom places. 
/// Protocol is a `prefix://` of filepath.
/// 
/// Examples: `file://dir1/dir2/myfile` - protocol is `file`; `https://www.github.com` - protocol is https.
/// 
/// In order to add your own protocol, just use:
/// ```rust
/// use shader_loader::preprocessor::FileLoader;
/// let mut loader = FileLoader::new();
/// loader.add_protocol("res".to_owned(), load_from_my_resources);
/// 
/// let my_file = loader.load_file("res://foo").unwrap();
/// assert_eq!(my_file.text(), "baz");
/// 
/// fn load_from_my_resources(path: &str) -> Result<String, String> {
///     // Put your logic to load files here
///     if path == "foo" {
///         return Ok("baz".to_owned());    
///     } else {
///         return Err("File does not exists".to_owned())    
///     }
/// }
/// ```
pub struct FileLoader {
    protocols: Vec<(String, Box<Protocol>)>,
}

fn load_file(path: &str) -> Result<String, String> {
    let pathbuf = std::fs::canonicalize(path)
        .map_err(|err| format!("Path error {path}: {}", err.to_string()))?;

    std::fs::read_to_string(pathbuf)
        .map_err(|err| format!("File loading error (file {path}): {}", err.to_string()))
}

impl FileLoader {
    pub fn new() -> Self {
        FileLoader { 
            protocols: vec![("file".to_string(), Box::new(load_file))],
        }
    }

    pub fn add_protocol<T>(&mut self, protocol: String, loader: T) -> Result<(), &'static str>
        where T: 'static + Fn(&str) -> Result<String, String>
    {
        for p in self.protocols.iter() {
            if p.0.eq(&protocol) {
                return Err("Protocol is already added");
            }
        }

        self.protocols.push((protocol, Box::new(loader)));
        Ok(())
    }

    pub fn load_file(&self, path: &str) -> Result<FileIncludes, String> {
        self.load_file_inner(path, &mut HashSet::new())
    }

    pub fn load_file_inner(&self, path: &str, used_files: &mut HashSet<String>) -> Result<FileIncludes, String> {
        lazy_static::lazy_static! {
            static ref INCLUDE_REGEX: Regex =       Regex::new(r#"\s*(#(?:pragma)? ?include_once *[ <"](?P<filename>[^\n\r"<>]*)[>"\n\r]?)"#).unwrap();
        }

        let dirname = crate::Path::new(path).dirname();
        used_files.insert(path.to_owned());
        let file = self.basic_load_file(path)?;
        let mut includes = FileIncludes::new(&file, path.to_owned());
        let mut jobs_to_replace: Vec<(usize, String)> = vec![];


        for (line_id, line) in includes.lines.iter().enumerate() {
            if let Some(cap) = INCLUDE_REGEX.captures(line) {
                let filepath = cap.get(2).unwrap();
                let filepath = &line[filepath.start()..filepath.end()];
                
                let filepath_owned;
                if get_protocol_and_path(filepath).0.is_none() { // Relative path
                    filepath_owned = dirname.join(filepath).to_string();
                } else { // Absolute
                    filepath_owned = filepath.to_owned();
                }
                

                jobs_to_replace.push((line_id, filepath_owned));
            }
        }

        let mut line_offset = 0;
        for (line_id, filepath) in jobs_to_replace.into_iter() {
            if used_files.contains(&filepath) { 
                // If file is already included - we just ignore
                includes.lines[line_id + line_offset] = "".to_owned();
            } else {
                used_files.insert(filepath.clone());
                let new_includes = self.load_file_inner(&filepath, used_files)?;
                let offset = new_includes.lines.len() - 1;
                includes.replace_line_with_includes(line_id + line_offset, new_includes);
                line_offset += offset;
            }
        }

        Ok(includes)
    }

    /// Just loads file as is. No proccessing
    pub fn basic_load_file(&self, path: &str) -> Result<String, String> {
        let (protocol, filepath) = get_protocol_and_path(path);
        let protocol = protocol.unwrap_or("file");
        let protocol = self.get_protocol(protocol)
            .ok_or(format!("Unsupported protocol: {protocol} ({path})"))?;

        let text = protocol(filepath)?;
        if text.is_empty() {
            Err(format!("Empty files ({path}) are unsupported because of technical reasons, sorry :("))
        } else {
            Ok(text)
        }
    }

    pub fn get_protocol(&self, name: &str) -> Option<&Protocol> {
        for (p_name, protocol) in self.protocols.iter() {
            if name == p_name {
                return Some(protocol);
            }
        }
        None
    }
}

impl Default for FileLoader {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_protocol_and_path(path: &str) -> (Option<&str>, &str) {
    lazy_static::lazy_static! {
        static ref REGEX: Regex = Regex::new(r#"^(\w+):\/\/"#).unwrap();
    }

    if let Some(captures) = REGEX.captures(path) {
        let full_match = captures.get(0).unwrap();
        let protocol = captures.get(1).unwrap();

        (Some(&path[0..protocol.end()]), &path[(full_match.end())..])
    } else {
        (None, path)
    }
}