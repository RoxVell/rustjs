use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone)]
pub enum Source {
    Inline(InlineSource),
    File(FileSource),
}

impl Default for Source {
    fn default() -> Self {
        Self::Inline(InlineSource::default())
    }
}

impl Source {
    pub fn inline_source(code: String) -> Self {
        Self::Inline(InlineSource { code })
    }

    pub fn filename(&self) -> &str {
        match self {
            Source::File(source) => &source.path.to_str().unwrap(),
            Source::Inline(_) => "<inline>"
        }
    }

    pub fn code(&self) -> &str {
        match self {
            Source::File(source) => &source.code,
            Source::Inline(source) => &source.code,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct InlineSource {
    pub(crate) code: String,
}

#[derive(Debug, Clone)]
pub struct FileSource {
    pub path: PathBuf,
    pub code: String,
}

impl Default for FileSource {
    fn default() -> Self {
        todo!()
    }
}

impl FileSource {
    pub fn new(path: PathBuf) -> Self {
        let code = fs::read_to_string(&path)
            .expect("Should have been able to read the file");

        Self {
            path,
            code,
        }
    }
}
