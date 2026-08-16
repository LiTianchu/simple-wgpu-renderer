use crate::{
    ds::{
        model::{MaterialStore, TextureStore},
        wgpu_resource::WgpuObject,
    },
    io::path_resolver,
};

pub fn preload_diffuse_normal_specular(
    wgpu_object: &WgpuObject,
    material_store: &MaterialStore,
    texture_store: &mut TextureStore,
) -> anyhow::Result<()> {
    // preload textures
    for (mat_key, material_obj) in material_store.materials() {
        if let Some(diff_path) = material_obj.material.texture_set.diffuse_map_path.as_ref() {
            let diff_full_path = path_resolver::get_texture_full_path(mat_key.clone(), diff_path)?;

            let texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;
            let texture_label = format!("Diffuse Texture: {}", diff_path);
            texture_store
                .get_or_load_texture(
                    &wgpu_object.device,
                    &wgpu_object.queue,
                    &diff_full_path,
                    texture_format,
                    texture_label,
                )
                .ok_or(anyhow::anyhow!(
                    "Failed to load diffuse texture at preloading stage {}",
                    diff_full_path,
                ))?;
        }
        if let Some(norm_path) = material_obj.material.texture_set.normal_map_path.as_ref() {
            let norm_full_path = path_resolver::get_texture_full_path(mat_key.clone(), norm_path)?;

            let texture_format = wgpu::TextureFormat::Rgba8Unorm;
            let texture_label = format!("Normal Texture: {}", norm_path);
            texture_store
                .get_or_load_texture(
                    &wgpu_object.device,
                    &wgpu_object.queue,
                    &norm_full_path,
                    texture_format,
                    texture_label,
                )
                .ok_or(anyhow::anyhow!(
                    "Failed to load normal texture at preloading stage {}",
                    norm_full_path,
                ))?;
        }
        if let Some(spec_path) = material_obj.material.texture_set.specular_map_path.as_ref() {
            let spec_full_path = path_resolver::get_texture_full_path(mat_key.clone(), spec_path)?;

            let texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;
            let texture_label = format!("Specular Texture: {}", spec_path);
            texture_store
                .get_or_load_texture(
                    &wgpu_object.device,
                    &wgpu_object.queue,
                    &spec_full_path,
                    texture_format,
                    texture_label,
                )
                .ok_or(anyhow::anyhow!(
                    "Failed to load specular texture at preloading stage {}",
                    spec_full_path,
                ))?;
        }
    }
    Ok(())
}
