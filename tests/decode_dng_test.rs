mod support;

use rawpipe::demosaic::OpDemosaic;
use rawpipe::Pipeline;

fn sample() -> std::path::PathBuf {
    support::synthetic_cfa_dng()
}

#[test]
fn cfa_phase_follows_the_crop_offset() {
    let pipeline = Pipeline::new_from_file(sample()).unwrap();
    // the synthetic file has DefaultCropOrigin (2, 1); the odd row offset turns RGGB into GBRG
    assert_eq!(
        (pipeline.ops.to_float.crop_left % 2, pipeline.ops.to_float.crop_top % 2),
        (0, 1)
    );
    assert_eq!(pipeline.ops.demosaic.cfa, "GBRG");
}

#[test]
fn shift_phase_rotates_the_pattern() {
    let shifted = |x: usize, y: usize| {
        let mut op = OpDemosaic { cfa: "RGGB".to_string() };
        op.shift_phase(x, y);
        op.cfa
    };
    assert_eq!(shifted(0, 0), "RGGB");
    assert_eq!(shifted(2, 4), "RGGB");
    assert_eq!(shifted(1, 0), "GRBG");
    assert_eq!(shifted(0, 1), "GBRG");
    assert_eq!(shifted(1, 1), "BGGR");

    // wider patterns (x-trans is 6x2) are not 2x2 and are left untouched
    let mut xtrans = OpDemosaic { cfa: "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBR".to_string() };
    let before = xtrans.cfa.clone();
    xtrans.shift_phase(1, 1);
    assert_eq!(xtrans.cfa, before);
}

#[test]
fn dng_renders_a_non_black_image() {
    let mut pipeline = Pipeline::new_from_file(sample()).unwrap();
    let image = pipeline.output_8bit(None).unwrap();
    assert_eq!((image.width, image.height), (60, 62));

    let pixels = (image.data.len() / 3) as u64;
    let lit = image.data.chunks_exact(3).filter(|p| p.iter().any(|&v| v > 32)).count() as u64;
    assert!(lit * 2 > pixels, "only {} of {} pixels are lit", lit, pixels);

    let max = image.data.iter().cloned().max().unwrap();
    assert!(max > 200, "brightest pixel is only {}", max);

    let mean = |c: usize| {
        image.data.iter().skip(c).step_by(3).map(|&v| v as f32).sum::<f32>() / pixels as f32
    };
    let means = (mean(0), mean(1), mean(2));
    assert!(
        means.0 > 20.0 && means.1 > 20.0 && means.2 > 20.0,
        "channel means {:?} look like an empty frame",
        means
    );
}

#[test]
fn black_level_override_darkens_the_shadows() {
    let mean = |pipeline: &mut Pipeline| {
        let image = pipeline.output_8bit(None).unwrap();
        image.data.iter().map(|&v| v as f32).sum::<f32>() / image.data.len() as f32
    };
    let mut plain = Pipeline::new_from_file(sample()).unwrap();
    let plain_mean = mean(&mut plain);
    let mut dark = Pipeline::new_from_file(sample()).unwrap();
    // the synthetic file has BlackLevel 0 and plenty of headroom above 256
    dark.ops.to_float.black_levels = [256.0; 4];
    let dark_mean = mean(&mut dark);
    assert!(
        dark_mean < plain_mean * 0.8,
        "black 256 should visibly darken: {:?} vs {:?}",
        dark_mean,
        plain_mean
    );
}
