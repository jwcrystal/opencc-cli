use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum AppError {
    /// Unsupported conversion mode string
    UnsupportedMode(String),
    /// File not found
    FileNotFound(PathBuf),
    /// Directory not found
    DirNotFound(PathBuf),
    /// No input provided
    NoInput,
    /// Conflicting input sources
    ConflictingInput,
    /// Output directory not found
    OutputDirNotFound(PathBuf),
    /// Output path is not a directory (expected dir for multi-file/dir mode)
    OutputNotDir(PathBuf),
    /// Input and output paths are the same without --in-place
    InputOutputSame(PathBuf),
    /// In-place requires file or dir input, not text
    InPlaceRequiresFiles,
    /// --in-place and -o are mutually exclusive
    InPlaceAndOutput,
    /// Multiple files require -o directory or --in-place
    MultiFileNoOutput,
    /// Basename conflict when writing multiple files to output dir
    BasenameConflict(PathBuf),
    /// Output directory overlaps with input directory
    OutputOverlapsInput(PathBuf),
    /// Path has no file name component
    NoFileName(PathBuf),
    /// No matching files found in directory
    EmptyDir { dir: PathBuf, exts: String },
    /// IO error
    Io(std::io::Error),
    /// UTF-8 decoding error
    Utf8(std::str::Utf8Error),
    /// `OpenCC` conversion error
    OpenCC(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedMode(mode) => {
                write!(f, "error: unsupported mode '{mode}'. Available: s2t, t2s, s2tw, tw2s, s2hk, hk2s, s2twp, tw2sp, t2tw, tw2t, t2hk, hk2t, t2jp, jp2t")
            }
            Self::FileNotFound(path) => write!(f, "error: file not found: '{}'", path.display()),
            Self::DirNotFound(path) => {
                write!(f, "error: directory not found: '{}'", path.display())
            }
            Self::NoInput => write!(
                f,
                "error: no input provided. Use -t, -f, -d, or pipe stdin."
            ),
            Self::ConflictingInput => write!(f, "error: -t, -f, and -d are mutually exclusive."),
            Self::OutputDirNotFound(path) => {
                write!(f, "error: output directory not found: '{}'", path.display())
            }
            Self::OutputNotDir(path) => {
                write!(
                    f,
                    "error: output path is not a directory: '{}'",
                    path.display()
                )
            }
            Self::InputOutputSame(path) => {
                write!(
                    f,
                    "error: input and output are the same file: '{}'. Use --in-place to overwrite.",
                    path.display()
                )
            }
            Self::InPlaceRequiresFiles => {
                write!(f, "error: --in-place requires -f or -d input, not -t.")
            }
            Self::InPlaceAndOutput => write!(f, "error: --in-place and -o are mutually exclusive."),
            Self::MultiFileNoOutput => {
                write!(
                    f,
                    "error: multiple files require -o <directory> or --in-place."
                )
            }
            Self::BasenameConflict(path) => {
                write!(
                    f,
                    "error: basename conflict in output: '{}'",
                    path.display()
                )
            }
            Self::OutputOverlapsInput(path) => {
                write!(
                    f,
                    "error: output directory overlaps with input: '{}'",
                    path.display()
                )
            }
            Self::NoFileName(path) => {
                write!(f, "error: path has no file name: '{}'", path.display())
            }
            Self::EmptyDir { dir, exts } => {
                write!(
                    f,
                    "error: no matching files in '{}' (--ext: {})",
                    dir.display(),
                    exts
                )
            }
            Self::Io(e) => write!(f, "error: {e}"),
            Self::Utf8(e) => write!(f, "error: invalid UTF-8: {e}"),
            Self::OpenCC(e) => write!(f, "error: opencc: {e}"),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<std::str::Utf8Error> for AppError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Utf8(e)
    }
}

impl From<tempfile::PersistError> for AppError {
    fn from(e: tempfile::PersistError) -> Self {
        Self::Io(e.error)
    }
}
