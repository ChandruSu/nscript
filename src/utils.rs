pub mod io {
    use std::{fs, str::Chars};

    type SourceId = u32;

    #[derive(Debug)]
    pub struct Source {
        id: SourceId,
        src_origin: String,
        src_content: String,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Pos {
        pub offset: i32,
        pub column: i32,
        pub line: u32,
        pub line_start: u32,
        pub src_id: SourceId,
    }

    pub struct SourceManager {
        sources: Vec<Source>,
    }

    impl Source {
        pub fn id(&self) -> u32 {
            self.id
        }

        pub fn char_stream(&self) -> Chars {
            self.src_content.chars()
        }
    }

    impl SourceManager {
        pub fn new() -> Self {
            Self { sources: vec![] }
        }

        pub fn get_source(&self, id: u32) -> Option<&Source> {
            self.sources.get(id as usize)
        }

        pub fn load_source_file(&mut self, file_path: &str) -> Result<&Source, String> {
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    self.sources.push(Source {
                        id: self.sources.len() as u32,
                        src_origin: file_path.to_string(),
                        src_content: content,
                    });

                    Ok(self.sources.last().unwrap())
                }
                Err(_) => Err(format!("Cannot read file: '{}'", file_path)),
            }
        }
    }
}

pub mod error {
    use super::io;

    pub struct LexerError {
        pub msg: String,
        pub pos: io::Pos,
    }
}
