mod support;

use rawpipe::{ImageSource, Pipeline};
use rawler::{Orientation, RawImageData};
use support::{verify_real_sample, SampleExpectation};

// Generated 16x16 grayscale gradient, stored as IEEE-float LinearRaw. It has no
// WhiteLevel or DefaultCrop tags, so it also exercises the float defaults.
fn fixture() -> std::path::PathBuf {
    support::synthetic_float_dng()
}

#[test]
fn float_linear_dng_renders_a_gradient() {
    let mut pipeline = Pipeline::new_from_file(fixture()).unwrap();
    match &pipeline.context.image {
        ImageSource::Raw(raw) => {
            assert!(matches!(raw.data, RawImageData::Float(_)));
            assert!(raw.crop_area.is_none());
        }
        ImageSource::Raster(_) => panic!("expected a raw image"),
    }

    let image = pipeline.output_8bit(None).unwrap();
    assert_eq!((image.width, image.height), (16, 16));

    let pixel = |x: usize| {
        let i = (8 * 16 + x) * 3;
        (image.data[i], image.data[i + 1], image.data[i + 2])
    };
    let dark = pixel(0);
    let middle = pixel(8);
    let bright = pixel(15);

    for (edge, value) in [("left", dark), ("right", bright)] {
        let spread = value.0.max(value.1).max(value.2) - value.0.min(value.1).min(value.2);
        assert!(spread <= 20, "{} edge is not neutral: {:?}", edge, value);
    }
    assert!(dark.0 < 40, "left edge should be dark, got {:?}", dark);
    assert!(middle.0 > dark.0 + 50, "middle {:?} should be brighter than left {:?}", middle, dark);
    assert!(bright.0 > 220, "right edge should be bright, got {:?}", bright);
}

#[test]
fn float_cfa_dng_real_sample() {
    verify_real_sample(&SampleExpectation {
        sample: "sample1.float.dng",
        raw_dims: (5920, 3950),
        out_dims: (5796, 3870),
        cpp: 1,
        is_float: true,
        is_linear_raw: false,
        cfa: "RGGB",
        black_levels: [2047.0, 2047.0, 2048.0, 2047.0],
        white_levels: [15488.0; 4],
        orientation: Some(Orientation::Normal),
        preview: Some((5796, 3870)),
        thumbnail: Some((256, 171)),
        scale_max: (320, 320),
        scale_out: (320, 213),
        min_range: 150,
    });
}