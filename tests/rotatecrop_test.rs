use rawpipe::color_conversions::{u8_to_unit, u16_to_unit};
use rawpipe::rotatecrop::OpRotateCrop;
use rawpipe::{ImageOp, PixelBuffer, PipelineContext};
use std::sync::Arc;

fn setup() -> (Arc<PixelBuffer>, OpRotateCrop, PipelineContext) {
    let mut buffer = PixelBuffer::new(100, 100, 3, false);
    for (i, o) in buffer.data.chunks_exact_mut(1).enumerate() {
        o[0] = i as f32;
    }
    let op = OpRotateCrop::empty();
    (Arc::new(buffer), op, PipelineContext::mock(100, 100))
}

#[test]
fn crop_top() {
    let (buffer, mut op, context) = setup();
    op.crop_top = 0.1;
    let newbuf = op.run(&context, buffer.clone());
    assert_eq!(newbuf.height, 90);
    assert_eq!(newbuf.width, 100);
    assert_eq!(&newbuf.data[0], &buffer.data[100 * 10 * 3]);
}

#[test]
fn crop_bottom() {
    let (buffer, mut op, context) = setup();
    op.crop_bottom = 0.1;
    let newbuf = op.run(&context, buffer.clone());
    assert_eq!(newbuf.height, 90);
    assert_eq!(newbuf.width, 100);
    assert_eq!(&newbuf.data[0], &buffer.data[0]);
}

#[test]
fn crop_vertical() {
    let (buffer, mut op, context) = setup();
    op.crop_top = 0.1;
    op.crop_bottom = 0.1;
    let newbuf = op.run(&context, buffer.clone());
    assert_eq!(newbuf.height, 80);
    assert_eq!(newbuf.width, 100);
    assert_eq!(&newbuf.data[0], &buffer.data[100 * 10 * 3]);
}

#[test]
fn crop_left() {
    let (buffer, mut op, context) = setup();
    op.crop_left = 0.1;
    let newbuf = op.run(&context, buffer.clone());
    assert_eq!(newbuf.height, 100);
    assert_eq!(newbuf.width, 90);
    assert_eq!(&newbuf.data[0], &buffer.data[10 * 3]);
}

#[test]
fn crop_right() {
    let (buffer, mut op, context) = setup();
    op.crop_right = 0.1;
    let newbuf = op.run(&context, buffer.clone());
    assert_eq!(newbuf.height, 100);
    assert_eq!(newbuf.width, 90);
    assert_eq!(&newbuf.data[0], &buffer.data[0]);
}

#[test]
fn crop_horizontal() {
    let (buffer, mut op, context) = setup();
    op.crop_left = 0.1;
    op.crop_right = 0.1;
    let newbuf = op.run(&context, buffer.clone());
    assert_eq!(newbuf.height, 100);
    assert_eq!(newbuf.width, 80);
    assert_eq!(&newbuf.data[0], &buffer.data[10 * 3]);
}

#[test]
fn crop_horizontal_and_vertical() {
    let (buffer, mut op, context) = setup();
    op.crop_left = 0.1;
    op.crop_right = 0.1;
    op.crop_top = 0.1;
    op.crop_bottom = 0.1;
    let newbuf = op.run(&context, buffer.clone());
    assert_eq!(newbuf.height, 80);
    assert_eq!(newbuf.width, 80);
    assert_eq!(&newbuf.data[0], &buffer.data[100 * 10 * 3 + 10 * 3]);
}

#[test]
fn rotate_45() {
    let (buffer, mut op, context) = setup();
    op.rotation = 0.5;
    let newbuf = op.run(&context, buffer.clone());
    assert_eq!(newbuf.height, 141);
    assert_eq!(newbuf.width, 141);
}

#[test]
fn rotate_90() {
    let (buffer, mut op, context) = setup();
    op.rotation = 1.0;
    let newbuf = op.run(&context, buffer.clone());
    assert_eq!(newbuf.height, 100);
    assert_eq!(newbuf.width, 100);
}

// Same configuration, but with no remembered output size: input_size() stores the size
// it was handed, and output_size() would just echo it back instead of computing anything.
fn without_planned_output(op: &OpRotateCrop) -> OpRotateCrop {
    let mut fresh = OpRotateCrop::empty();
    fresh.crop_top = op.crop_top;
    fresh.crop_right = op.crop_right;
    fresh.crop_bottom = op.crop_bottom;
    fresh.crop_left = op.crop_left;
    fresh.rotation = op.rotation;
    fresh
}

// input_size() reconstructs the pre-rotation size from the rotated one, and that is only defined
// up to the rounding of the integer sizes. Measured over this sweep (aspect ratios up to
// ~10000:1) the roundtrip is never more than one pixel off.
fn within_one_pixel(result: (usize, usize), expected: (usize, usize)) -> bool {
    result.0.abs_diff(expected.0) <= 1 && result.1.abs_diff(expected.1) <= 1
}

#[test]
fn roundtrip_transform() {
    let mut config = OpRotateCrop::empty();
    // sizes start at 1: a zero-sized image is not a pipeline input
    for dim in (1..10000).step_by(89) {
        for crop1 in (0..u16::MAX).step_by(97) {
            for crop2 in (0..u16::MAX).step_by(101) {
                config.crop_top = u16_to_unit(crop1);
                config.crop_right = u16_to_unit(crop1);
                config.crop_bottom = u16_to_unit(crop2);
                config.crop_left = u16_to_unit(crop2);
                let from = (dim, dim);
                let mut reverse = without_planned_output(&config);
                let intermediate = reverse.input_size(from.0, from.1);
                let mut forward = without_planned_output(&config);
                let result = forward.output_size(intermediate.0, intermediate.1);
                assert_eq!(
                    result,
                    from,
                    "Got {:?}->{:?}->{:?} crops ({:.3}/{:.3}/{:.3}/{:.3})",
                    from,
                    intermediate,
                    result,
                    config.crop_top,
                    config.crop_right,
                    config.crop_bottom,
                    config.crop_left
                );
            }
        }
    }
}

#[test]
fn roundtrip_transform_rotation() {
    let mut config = OpRotateCrop::empty();
    for width in (1..10000).step_by(89) {
        for height in (1..10000).step_by(97) {
            for rotation in 0..=u8::MAX {
                config.rotation = u8_to_unit(rotation);
                let from = (width, height);
                // output_size() records the aspect ratio input_size() needs, so those two share an op
                let mut calc = without_planned_output(&config);
                let inter1 = calc.output_size(from.0, from.1);
                let inter2 = calc.input_size(inter1.0, inter1.1);
                let mut forward = without_planned_output(&config);
                let result = forward.output_size(inter2.0, inter2.1);
                assert!(
                    within_one_pixel(result, inter1),
                    "Got {:?}->{:?}->{:?}->{:?} crops ({:.3}/{:.3}/{:.3}/{:.3})",
                    from,
                    inter1,
                    inter2,
                    result,
                    config.crop_top,
                    config.crop_right,
                    config.crop_bottom,
                    config.crop_left
                );
            }
        }
    }
}

