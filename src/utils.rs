pub mod io {
    use crate::error;
    use std::{cmp, fs, str::Chars};

    type SourceId = u32;

    #[derive(Debug)]
    pub struct Source {
        id: SourceId,
        src_origin: String,
        src_content: String,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
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

        pub fn get_origin(&self) -> &String {
            &self.src_origin
        }
    }

    impl SourceManager {
        pub fn new() -> Self {
            Self { sources: vec![] }
        }

        pub fn get_source(&self, id: u32) -> Option<&Source> {
            self.sources.get(id as usize)
        }

        pub fn load_source_file(&mut self, file_path: &str) -> Result<&Source, error::Error> {
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    self.sources.push(Source {
                        id: self.sources.len() as u32,
                        src_content: content.replace("\t", "    "),
                        src_origin: fs::canonicalize(file_path)
                            .map(|p| {
                                p.into_os_string()
                                    .into_string()
                                    .unwrap()
                                    .trim_start_matches("\\\\?\\")
                                    .to_string()
                            })
                            .unwrap_or(file_path.to_string()),
                    });

                    Ok(self.sources.last().unwrap())
                }
                Err(_) => Err(error::Error::file_read_error(file_path)),
            }
        }

        pub fn get_line(&self, pos: &Pos) -> Option<String> {
            match self.get_source(pos.src_id) {
                Some(src) => {
                    let idx = pos.line_start as usize;
                    let line_len = src.src_content[idx..]
                        .find('\n')
                        .unwrap_or(src.src_content.len() - idx - 1);
                    let line_end = cmp::min(idx + cmp::min(line_len, 200), src.src_content.len());
                    Some(src.src_content[idx..line_end].to_string())
                }
                None => None,
            }
        }
    }
}
