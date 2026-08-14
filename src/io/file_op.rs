use anyhow::Context;
use std::fs::{self, DirEntry};
use std::path::PathBuf;

pub fn load_binary(file_path: impl Into<String>) -> anyhow::Result<Vec<u8>> {
    let file_path = file_path.into();
    let bytes = fs::read(file_path.clone())
        .with_context(|| format!("Failed to read binary file: {}", file_path))?;
    Ok(bytes)
}

pub fn get_files_by_type_recur(
    path_str: impl Into<String>,
    file_type: &str,
) -> anyhow::Result<Vec<PathBuf>> {
    let path_str = path_str.into();
    let mut paths: Vec<PathBuf> = Vec::new();
    println!("Searching path: {}", path_str.clone());
    get_files_by_type_helper(path_str, &mut paths, file_type)?;

    return Ok(paths);
}

fn get_files_by_type_helper(
    path_str: impl Into<String>,
    paths: &mut Vec<PathBuf>,
    target_file_type: &str,
) -> anyhow::Result<()> {
    let path_str = path_str.into();

    let path: PathBuf = PathBuf::from(path_str.clone());
    if path.is_dir() {
        let read_dir = path
            .read_dir()
            .with_context(|| format!("Failed to read dir: {}", path.to_string_lossy()))?;

        for (i, entry_result) in read_dir.enumerate() {
            let entry: DirEntry = entry_result.with_context(|| {
                format!(
                    "Failed to read dir entry {} - {}",
                    path.to_string_lossy(),
                    i
                )
            })?;

            match entry.file_type() {
                Ok(f_type) => {
                    if f_type.is_dir() {
                        let entry_path = entry.path();
                        println!(
                            "Found sub folder at {}, searching...",
                            entry_path.to_string_lossy()
                        );

                        get_files_by_type_helper(
                            entry_path.to_string_lossy(),
                            paths,
                            target_file_type,
                        )?;
                    } else {
                        let path_buf = entry.path();
                        if path_buf.extension().and_then(|ext| ext.to_str())
                            == Some(target_file_type)
                        {
                            paths.push(path_buf);
                        }
                    }
                }
                Err(_) => {}
            }
        }
    } else {
        if path.extension().and_then(|ext| ext.to_str()) == Some(target_file_type) {
            paths.push(path);
        }
    }
    Ok(())
}
