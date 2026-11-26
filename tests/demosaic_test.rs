use rawpipe::demosaic::OpDemosaic;
use rawpipe::{ImageOp, PixelBuffer, PipelineContext};
use std::sync::Arc;

fn run_demosaic(pattern: &str, width: usize, height: usize, target_width: usize, target_height: usize) -> PixelBuffer {
    let op = OpDemosaic { cfa: pattern.to_string() };
    let mut context = PipelineContext::mock(width as u32, height as u32);
    context.settings.demosaic_width = target_width;
    context.settings.demosaic_height = target_height;

    let mut buffer = PixelBuffer::new(width, height, 1, false);
    for (i, value) in buffer.data.iter_mut().enumerate() {
        *value = (i % 997) as f32 / 997.0;
    }

    (*op.run(&context, Arc::new(buffer))).clone()
}

fn assert_demosaiced(buffer: &PixelBuffer, width: usize, height: usize) {
    assert_eq!((buffer.width, buffer.height, buffer.channels), (width, height, 4));
    assert!(buffer.data.iter().all(|v| v.is_finite()));
    assert!(buffer.data.iter().any(|v| *v > 0.0));
}

#[test]
fn demosaics_a_2x8_pattern() {
    // 16 characters are the 2x8 pattern shape (Canon Pro70-era files)
    let out = run_demosaic("RGGBRGGBRGGBRGGB", 64, 64, 64, 64);
    assert_demosaiced(&out, 64, 64);
}

#[test]
fn demosaics_a_12x12_pattern_at_full_size() {
    let pattern = "RGGB".repeat(36);
    let out = run_demosaic(&pattern, 120, 120, 120, 120);
    assert_demosaiced(&out, 120, 120);
}

#[test]
fn scales_down_a_12x12_pattern() {
    // scale is exactly the 12x12 minimum, so this takes the scaled demosaic path
    let pattern = "RGGB".repeat(36);
    let out = run_demosaic(&pattern, 120, 120, 10, 10);
    assert_demosaiced(&out, 10, 10);
}