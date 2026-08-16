pub fn get_texture_full_path(
    material_full_path: impl Into<String>,
    texture_relative_path: impl Into<String>,
) -> anyhow::Result<String> {
    let material_full_path: String = material_full_path.into();
    let texture_relative_path: String = texture_relative_path.into();
    let material_path = std::path::Path::new(&material_full_path);
    let material_dir = material_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory of material path"))?;

    let texture_full_path = material_dir.join(texture_relative_path);

    let texture_full_path_str = texture_full_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert texture path to string"))?;

    Ok(texture_full_path_str.to_string())
}
