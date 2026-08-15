use std::sync::mpsc;
pub fn buffer_slice_to_byte_array(
    buffer_slice: wgpu::BufferSlice,
    submission_index: wgpu::SubmissionIndex,
    device: &wgpu::Device,
    output_height: u32,
    padded_byte_per_row: u32,
    unpadded_bytes_per_row: u32,
) -> anyhow::Result<Vec<u8>> {
    // use sender receiver pattern to receive the result
    let (sender, receiver) = mpsc::channel();

    buffer_slice.map_async(
        wgpu::MapMode::Read,
        move |result: Result<(), wgpu::BufferAsyncError>| {
            let _ = sender.send(result);
        },
    );

    // waits for GPU work and invokes pending mapping callbacks
    // need to poll as CPU need to wait for the async mapping result
    device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission_index),
        timeout: None,
    })?;

    //receive the result and throw error if needed
    let _ = receiver
        .recv()
        .map_err(|_| anyhow::anyhow!("Buffer mapping callback was dropped"))?;

    let mapped_data: wgpu::BufferView = buffer_slice.get_mapped_range()?;

    // use unpadded bytes for output to trim the gpu padded bytes
    let mut output_img_pixels: Vec<u8> =
        vec![0_u8; (unpadded_bytes_per_row * output_height) as usize];

    // copy row-by-row to the output img pixel array
    for (source_row, dest_row) in mapped_data
        .chunks_exact(padded_byte_per_row as usize)
        .zip(output_img_pixels.chunks_exact_mut(unpadded_bytes_per_row as usize))
    {
        dest_row.copy_from_slice(&source_row[0..unpadded_bytes_per_row as usize]);
    }

    // clean up
    drop(mapped_data);
    Ok(output_img_pixels)
}

pub fn write_texture_rgba(
    queue: &wgpu::Queue,
    write_target_texture: &wgpu::Texture,
    img_pixel_dimensions: (u32, u32),
    rgba_data: &[u8],
) {
    let texture_size = wgpu::Extent3d {
        width: img_pixel_dimensions.0,
        height: img_pixel_dimensions.1,
        depth_or_array_layers: 1,
    };

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &write_target_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba_data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * img_pixel_dimensions.0),
            rows_per_image: Some(img_pixel_dimensions.1),
        },
        texture_size,
    );
}
