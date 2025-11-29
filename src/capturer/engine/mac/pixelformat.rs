use std::time::SystemTime;

use screencapturekit::cm::CMSampleBuffer;

use crate::frame::{
    convert_bgra_to_rgb, get_cropped_data, remove_alpha_channel, BGRAFrame, BGRFrame, RGBFrame,
    YUVFrame,
};

pub unsafe fn create_yuv_frame(
    sample_buffer: &CMSampleBuffer,
    display_time: SystemTime,
) -> Option<YUVFrame> {
    let image_buffer = sample_buffer.get_image_buffer()?;

    // Lock the pixel buffer - guard unlocks automatically on drop
    let _guard = image_buffer.lock_base_address(false).ok()?;

    let width = image_buffer.width();
    let height = image_buffer.height();

    if width == 0 || height == 0 {
        return None;
    }

    // For YUV420, plane 0 is luminance, plane 1 is chrominance
    let luminance_stride = image_buffer.get_bytes_per_row_of_plane(0);
    let luminance_height = image_buffer.get_height_of_plane(0);
    let luminance_base_address = image_buffer.get_base_address_of_plane(0)?;

    let luminance_bytes = unsafe {
        std::slice::from_raw_parts(luminance_base_address, luminance_stride * luminance_height)
    }
    .to_vec();

    let chrominance_stride = image_buffer.get_bytes_per_row_of_plane(1);
    let chrominance_height = image_buffer.get_height_of_plane(1);
    let chrominance_base_address = image_buffer.get_base_address_of_plane(1)?;

    let chrominance_bytes = unsafe {
        std::slice::from_raw_parts(
            chrominance_base_address,
            chrominance_stride * chrominance_height,
        )
    }
    .to_vec();

    Some(YUVFrame {
        display_time,
        width: width as i32,
        height: height as i32,
        luminance_bytes,
        luminance_stride: luminance_stride as i32,
        chrominance_bytes,
        chrominance_stride: chrominance_stride as i32,
    })
}

pub unsafe fn create_bgr_frame(
    sample_buffer: &CMSampleBuffer,
    display_time: SystemTime,
) -> Option<BGRFrame> {
    let image_buffer = sample_buffer.get_image_buffer()?;

    let _guard = image_buffer.lock_base_address(false).ok()?;

    let width = image_buffer.width();
    let height = image_buffer.height();

    if width == 0 || height == 0 {
        return None;
    }

    let stride = image_buffer.get_bytes_per_row_of_plane(0);
    let base_address = image_buffer.get_base_address_of_plane(0)?;

    let bytes = unsafe { std::slice::from_raw_parts(base_address, stride * height) }.to_vec();

    let cropped_data = get_cropped_data(bytes, (stride / 4) as i32, height as i32, width as i32);

    Some(BGRFrame {
        display_time,
        width: width as i32,
        height: height as i32,
        data: remove_alpha_channel(cropped_data),
    })
}

pub unsafe fn create_bgra_frame(
    sample_buffer: &CMSampleBuffer,
    display_time: SystemTime,
) -> Option<BGRAFrame> {
    let image_buffer = sample_buffer.get_image_buffer()?;

    let _guard = image_buffer.lock_base_address(false).ok()?;

    let width = image_buffer.width();
    let height = image_buffer.height();

    if width == 0 || height == 0 {
        return None;
    }

    let stride = image_buffer.get_bytes_per_row_of_plane(0);
    let base_address = image_buffer.get_base_address_of_plane(0)?;

    let mut data: Vec<u8> = vec![];

    let bytes = unsafe { std::slice::from_raw_parts(base_address, stride * height) };

    for i in 0..height {
        let base = i * stride;
        data.extend_from_slice(&bytes[base as usize..(base + 4 * width) as usize]);
    }

    Some(BGRAFrame {
        display_time,
        width: width as i32,
        height: height as i32,
        data,
    })
}

pub unsafe fn create_rgb_frame(
    sample_buffer: &CMSampleBuffer,
    display_time: SystemTime,
) -> Option<RGBFrame> {
    let image_buffer = sample_buffer.get_image_buffer()?;

    let _guard = image_buffer.lock_base_address(false).ok()?;

    let width = image_buffer.width();
    let height = image_buffer.height();

    if width == 0 || height == 0 {
        return None;
    }

    let stride = image_buffer.get_bytes_per_row_of_plane(0);
    let base_address = image_buffer.get_base_address_of_plane(0)?;

    let bytes = unsafe { std::slice::from_raw_parts(base_address, stride * height) }.to_vec();

    let cropped_data = get_cropped_data(bytes, (stride / 4) as i32, height as i32, width as i32);

    Some(RGBFrame {
        display_time,
        width: width as i32,
        height: height as i32,
        data: convert_bgra_to_rgb(cropped_data),
    })
}
