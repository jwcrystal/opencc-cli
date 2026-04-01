use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::convert::convert_text;
use crate::error::AppError;
use ferrous_opencc::OpenCC;

/// Read stdin as a string.
pub fn read_stdin() -> Result<String, AppError> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

/// Read a single file as UTF-8 string.
pub fn read_file(path: &Path) -> Result<String, AppError> {
    if !path.exists() {
        return Err(AppError::FileNotFound(path.to_path_buf()));
    }
    Ok(fs::read_to_string(path)?)
}

/// Collect all files in a directory recursively, filtered by extensions.
/// Symlinks are skipped to prevent directory traversal attacks.
pub fn collect_dir_files(dir: &Path, exts: &[&str]) -> Result<Vec<PathBuf>, AppError> {
    if !dir.is_dir() {
        return Err(AppError::DirNotFound(dir.to_path_buf()));
    }

    let ext_set: Vec<String> = exts.iter().map(|e| e.to_lowercase()).collect();
    let mut files = Vec::new();

    collect_files_recursive(dir, &ext_set, &mut files)?;

    if files.is_empty() {
        return Err(AppError::EmptyDir {
            dir: dir.to_path_buf(),
            exts: exts.join(","),
        });
    }

    files.sort();
    Ok(files)
}

fn collect_files_recursive(
    current: &Path,
    ext_set: &[String],
    files: &mut Vec<PathBuf>,
) -> Result<(), AppError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();

        // Use symlink_metadata to detect symlinks without following them
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.is_symlink() {
            continue;
        }

        if path.is_dir() {
            collect_files_recursive(&path, ext_set, files)?;
        } else {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if ext_set.contains(&ext) {
                files.push(path);
            }
        }
    }
    Ok(())
}

/// Process a single file: convert and write to output file or in-place.
pub fn process_file(
    converter: &OpenCC,
    input_path: &Path,
    output_path: Option<&Path>,
    in_place: bool,
) -> Result<(), AppError> {
    let content = read_file(input_path)?;
    let result = convert_text(converter, &content);

    match (output_path, in_place) {
        (Some(out), false) => {
            // Input == output check
            let canonical_in = input_path.canonicalize().ok();
            let canonical_out = out.canonicalize().ok();
            if canonical_in.is_some() && canonical_in == canonical_out {
                return Err(AppError::InputOutputSame(input_path.to_path_buf()));
            }
            fs::write(out, &result)?;
        }
        (None, true) => {
            // In-place: write to a unique temp file then rename (atomic)
            let parent = input_path.parent().unwrap_or(Path::new("."));
            let mut tmp = tempfile::Builder::new()
                .prefix(".opencc_tmp_")
                .rand_bytes(6)
                .tempfile_in(parent)?;
            tmp.write_all(result.as_bytes())?;
            tmp.persist(input_path)?;
        }
        _ => unreachable!("process_file: invalid output/in_place combination"),
    }

    Ok(())
}

/// Check whether `output_dir` is inside `input_dir` or vice versa.
pub fn dirs_overlap(input_dir: &Path, output_dir: &Path) -> bool {
    let Ok(canonical_in) = input_dir.canonicalize() else {
        return false;
    };
    let Ok(canonical_out) = output_dir.canonicalize() else {
        return false;
    };
    canonical_in == canonical_out
        || canonical_in.starts_with(&canonical_out)
        || canonical_out.starts_with(&canonical_in)
}

/// Process directory: convert all matched files, preserving relative path structure.
pub fn process_dir(
    converter: &OpenCC,
    input_dir: &Path,
    output_dir: &Path,
    exts: &[&str],
    in_place: bool,
) -> Result<(), AppError> {
    let files = collect_dir_files(input_dir, exts)?;

    if in_place {
        for file_path in &files {
            process_file(converter, file_path, None, true)?;
        }
    } else {
        if !output_dir.exists() {
            return Err(AppError::OutputDirNotFound(output_dir.to_path_buf()));
        }
        if !output_dir.is_dir() {
            return Err(AppError::OutputNotDir(output_dir.to_path_buf()));
        }
        if dirs_overlap(input_dir, output_dir) {
            return Err(AppError::OutputOverlapsInput(output_dir.to_path_buf()));
        }

        let mut seen = std::collections::HashSet::new();
        for file_path in &files {
            let relative = file_path.strip_prefix(input_dir).unwrap_or(file_path);
            let key = relative.to_string_lossy().to_string();
            if !seen.insert(key) {
                return Err(AppError::BasenameConflict(relative.to_path_buf()));
            }
        }

        for file_path in &files {
            let relative = file_path.strip_prefix(input_dir).unwrap_or(file_path);
            let out_path = output_dir.join(relative);

            // Create parent directories
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }

            process_file(converter, file_path, Some(&out_path), false)?;
        }
    }

    Ok(())
}
