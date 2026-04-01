use std::fs;
use std::io::{self, Read};
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
    let bytes = fs::read(path)?;
    let content = std::str::from_utf8(&bytes)?;
    Ok(content.to_string())
}

/// Collect all files in a directory recursively, filtered by extensions.
pub fn collect_dir_files(dir: &Path, exts: &[&str]) -> Result<Vec<PathBuf>, AppError> {
    if !dir.is_dir() {
        return Err(AppError::DirNotFound(dir.to_path_buf()));
    }

    let ext_set: Vec<String> = exts.iter().map(|e| e.to_lowercase()).collect();
    let mut files = Vec::new();

    collect_files_recursive(dir, dir, &ext_set, &mut files)?;

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
    base: &Path,
    current: &Path,
    ext_set: &[String],
    files: &mut Vec<PathBuf>,
) -> Result<(), AppError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(base, &path, ext_set, files)?;
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

/// Process a single file: convert and write to stdout, output file, or in-place.
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
            // In-place: write to temp file then rename
            let tmp_path = input_path.with_extension("opencc_tmp");
            fs::write(&tmp_path, &result)?;
            fs::rename(&tmp_path, input_path)?;
        }
        _ => unreachable!("process_file: invalid output/in_place combination"),
    }

    Ok(())
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

    if !in_place {
        if !output_dir.exists() {
            return Err(AppError::OutputDirNotFound(output_dir.to_path_buf()));
        }
        if !output_dir.is_dir() {
            return Err(AppError::OutputNotDir(output_dir.to_path_buf()));
        }

        // Check basename conflicts
        check_basename_conflicts(&files, output_dir)?;

        for file_path in &files {
            let relative = file_path.strip_prefix(input_dir).unwrap_or(file_path);
            let out_path = output_dir.join(relative);

            // Create parent directories
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }

            process_file(converter, file_path, Some(&out_path), false)?;
        }
    } else {
        for file_path in &files {
            process_file(converter, file_path, None, true)?;
        }
    }

    Ok(())
}

/// Check for basename conflicts in output directory.
fn check_basename_conflicts(files: &[PathBuf], output_dir: &Path) -> Result<(), AppError> {
    let mut seen = std::collections::HashSet::new();
    for file_path in files {
        let relative = file_path.strip_prefix(file_path.parent().unwrap()).unwrap();
        let out_path = output_dir.join(relative);
        let key = out_path.to_string_lossy().to_string();
        if !seen.insert(key.clone()) {
            return Err(AppError::BasenameConflict(PathBuf::from(key)));
        }
    }
    Ok(())
}
