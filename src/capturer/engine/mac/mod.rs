use screencapturekit::{
    cm::{CMSampleBuffer, CMTime},
    shareable_content::{SCShareableContent, SCWindow},
    stream::{
        configuration::{PixelFormat, SCStreamConfiguration},
        content_filter::SCContentFilter,
        delegate_trait::SCStreamDelegateTrait,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
        sc_stream::SCStream,
    },
};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::{cmp, sync::Arc};

use crate::frame::{AudioFormat, AudioFrame, Frame, FrameType, VideoFrame};
use crate::targets::Target;
use crate::{
    capturer::{Area, Options, Point, Resolution, Size},
    frame::BGRAFrame,
    targets,
};

use super::ChannelItem;

pub(crate) mod ext;
mod pixel_buffer;
mod pixelformat;

pub struct ErrorHandler {
    error_flag: Arc<AtomicBool>,
}

impl SCStreamDelegateTrait for ErrorHandler {
    fn stream_did_stop(&self, error: Option<String>) {
        if let Some(err) = error {
            eprintln!("Screen capture error occurred: {}", err);
        } else {
            eprintln!("Screen capture error occurred.");
        }
        self.error_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

pub struct Capturer {
    pub tx: mpsc::Sender<ChannelItem>,
}

impl SCStreamOutputTrait for Capturer {
    fn did_output_sample_buffer(
        &self,
        sample_buffer: CMSampleBuffer,
        output_type: SCStreamOutputType,
    ) {
        let _ = self.tx.send((sample_buffer, output_type));
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum CreateCapturerError {
    #[error("{0}")]
    OtherNative(String),
    #[error("Window with title '{0}' not found")]
    WindowNotFound(String),
    #[error("Display with title '{0}' not found")]
    DisplayNotFound(String),
}

pub(crate) fn create_capturer(
    options: &Options,
    tx: mpsc::Sender<ChannelItem>,
    error_flag: Arc<AtomicBool>,
) -> Result<(Arc<Capturer>, Arc<ErrorHandler>, SCStream), CreateCapturerError> {
    // If no target is specified, capture the main display
    let target = options
        .target
        .clone()
        .unwrap_or_else(|| Target::Display(targets::get_main_display()));

    let shareable_content = SCShareableContent::get().map_err(|e| {
        CreateCapturerError::OtherNative(format!("Failed to get shareable content: {:?}", e))
    })?;

    let filter = match target {
        Target::Window(window) => {
            let windows = shareable_content.windows();

            // Get SCWindow from window id
            let sc_window = windows
                .iter()
                .find(|sc_win| sc_win.window_id() == window.id)
                .ok_or_else(|| CreateCapturerError::WindowNotFound(window.title))?;

            // Return a DesktopIndependentWindow
            SCContentFilter::builder().window(sc_window).build()
        }
        Target::Display(display) => {
            let displays = shareable_content.displays();
            // Get SCDisplay from display id
            let sc_display = displays
                .iter()
                .find(|sc_dis| sc_dis.display_id() == display.raw_handle)
                .ok_or_else(|| CreateCapturerError::DisplayNotFound(display.title))?;

            match &options.excluded_targets {
                None => SCContentFilter::builder()
                    .display(sc_display)
                    .exclude_windows(&[])
                    .build(),
                Some(excluded_targets) => {
                    let windows = shareable_content.windows();
                    let excluded_windows: Vec<&SCWindow> = windows
                        .iter()
                        .filter(|window| {
                            excluded_targets
                                .iter()
                                .any(|excluded_target| match excluded_target {
                                    Target::Window(excluded_window) => {
                                        excluded_window.id == window.window_id()
                                    }
                                    _ => false,
                                })
                        })
                        .collect();

                    SCContentFilter::builder()
                        .display(sc_display)
                        .exclude_windows(&excluded_windows)
                        .build()
                }
            }
        }
    };

    let crop_area = get_crop_area(options);

    let source_rect = screencapturekit::cg::CGRect {
        x: crop_area.origin.x,
        y: crop_area.origin.y,
        width: crop_area.size.width,
        height: crop_area.size.height,
    };

    let pixel_format = match options.output_type {
        FrameType::YUVFrame => PixelFormat::YCbCr_420v,
        FrameType::BGR0 => PixelFormat::BGRA,
        FrameType::RGB => PixelFormat::BGRA,
        FrameType::BGRAFrame => PixelFormat::BGRA,
    };

    let [width, height] = get_output_frame_size(options);

    let mut stream_config = SCStreamConfiguration::default();
    stream_config.set_width(width);
    stream_config.set_height(height);
    stream_config.set_source_rect(source_rect);
    stream_config.set_pixel_format(pixel_format);
    stream_config.set_shows_cursor(options.show_cursor);
    stream_config.set_minimum_frame_interval(&CMTime {
        value: 1,
        timescale: options.fps as i32,
        epoch: 0,
        flags: 1, // CMTimeFlags::VALID
    });
    stream_config.set_captures_audio(options.captures_audio);

    let error_handler = Arc::new(ErrorHandler { error_flag });
    let capturer = Arc::new(Capturer { tx });

    let mut stream = SCStream::new_with_delegate(
        &filter,
        &stream_config,
        ErrorHandler {
            error_flag: error_handler.error_flag.clone(),
        },
    );

    if options.captures_audio {
        stream.add_output_handler(
            Capturer {
                tx: capturer.tx.clone(),
            },
            SCStreamOutputType::Audio,
        );
    }

    stream.add_output_handler(
        Capturer {
            tx: capturer.tx.clone(),
        },
        SCStreamOutputType::Screen,
    );

    Ok((capturer, error_handler, stream))
}

pub fn show_target_picker() {
    // Show picker on macOS - can be implemented using SCContentSharingPicker
    // This requires macos_14_0 feature
}

pub fn get_output_frame_size(options: &Options) -> [u32; 2] {
    let target = options
        .target
        .clone()
        .unwrap_or_else(|| Target::Display(targets::get_main_display()));

    let scale_factor = targets::get_scale_factor(&target);
    let source_rect = get_crop_area(options);

    // Calculate the output height & width based on the required resolution
    // Output width and height need to be multiplied by scale (or dpi)
    let mut output_width = (source_rect.size.width as u32) * (scale_factor as u32);
    let mut output_height = (source_rect.size.height as u32) * (scale_factor as u32);
    // 1200x800
    match options.output_resolution {
        Resolution::Captured => {}
        _ => {
            let [resolved_width, resolved_height] = options
                .output_resolution
                .value((source_rect.size.width as f32) / (source_rect.size.height as f32));
            // 1280 x 853
            output_width = cmp::min(output_width, resolved_width);
            output_height = cmp::min(output_height, resolved_height);
        }
    }

    output_width -= output_width % 2;
    output_height -= output_height % 2;

    [output_width, output_height]
}

pub fn get_crop_area(options: &Options) -> Area {
    let target = options
        .target
        .clone()
        .unwrap_or_else(|| Target::Display(targets::get_main_display()));

    let (width, height) = targets::get_target_dimensions(&target);

    options
        .crop_area
        .as_ref()
        .map(|val| {
            let input_width = val.size.width + (val.size.width % 2.0);
            let input_height = val.size.height + (val.size.height % 2.0);

            Area {
                origin: Point {
                    x: val.origin.x,
                    y: val.origin.y,
                },
                size: Size {
                    width: input_width,
                    height: input_height,
                },
            }
        })
        .unwrap_or_else(|| Area {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: width as f64,
                height: height as f64,
            },
        })
}

pub fn process_sample_buffer(
    sample: CMSampleBuffer,
    of_type: SCStreamOutputType,
    output_type: FrameType,
) -> Option<Frame> {
    let system_time = std::time::SystemTime::now();

    // Get presentation timestamp from sample buffer
    let frame_cm_time = sample.get_presentation_timestamp();

    // Convert CMTime to SystemTime
    // CMTime uses a timescale, we need to convert to nanoseconds
    let timescale = frame_cm_time.timescale;
    let value = frame_cm_time.value;

    // Convert to nanoseconds: (value * 1_000_000_000) / timescale
    let nanos = (value as i64).saturating_mul(1_000_000_000) / timescale as i64;

    // Calculate frame SystemTime
    let frame_system_time = if nanos >= 0 {
        system_time + std::time::Duration::from_nanos(nanos as u64)
    } else {
        system_time - std::time::Duration::from_nanos((-nanos) as u64)
    };

    match of_type {
        SCStreamOutputType::Screen => {
            // Check frame status
            if let Some(status) = sample.get_frame_status() {
                // Status Complete means valid frame, others are incomplete
                if status != screencapturekit::cm::SCFrameStatus::Complete {
                    // Incomplete frame
                    if let FrameType::BGRAFrame = output_type {
                        return Some(Frame::Video(VideoFrame::BGRA(BGRAFrame {
                            display_time: frame_system_time,
                            width: 0,
                            height: 0,
                            data: vec![],
                        })));
                    }
                    return None;
                }
            }

            unsafe {
                return match output_type {
                    FrameType::YUVFrame => {
                        pixelformat::create_yuv_frame(&sample, frame_system_time)
                            .map(|yuvframe| Frame::Video(VideoFrame::YUVFrame(yuvframe)))
                    }
                    FrameType::RGB => pixelformat::create_rgb_frame(&sample, frame_system_time)
                        .map(|rgbframe| Frame::Video(VideoFrame::RGB(rgbframe))),
                    FrameType::BGR0 => pixelformat::create_bgr_frame(&sample, frame_system_time)
                        .map(|bgrframe| Frame::Video(VideoFrame::BGR0(bgrframe))),
                    FrameType::BGRAFrame => {
                        pixelformat::create_bgra_frame(&sample, frame_system_time)
                            .map(|bgraframe| Frame::Video(VideoFrame::BGRA(bgraframe)))
                    }
                };
            }
        }
        SCStreamOutputType::Audio => {
            // Extract audio data from CMSampleBuffer
            let audio_buffer_list = sample.get_audio_buffer_list()?;
            let mut bytes = Vec::<u8>::new();

            // Iterate through audio buffers
            for buffer in audio_buffer_list.iter() {
                let data = buffer.data();
                bytes.extend_from_slice(data);
            }

            return Some(Frame::Audio(AudioFrame::new(
                AudioFormat::F32,
                2,
                false,
                bytes,
                sample.get_num_samples(),
                48_000,
                frame_system_time,
            )));
        }
        _ => None,
    }
}
