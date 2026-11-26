mod support;

use rawpipe::Pipeline;

// (sample file, expected output width and height)
const SAMPLES: &[(&str, usize, usize)] = &[
    ("sample1.3fr", 7216, 5412),
    ("sample1.arw", 2768, 1848),
    ("sample1.cr2", 8688, 5792),
    ("sample1.cr3", 6000, 4000),
    ("sample1.crw", 2224, 1456),
    ("sample1.dcr", 3032, 2008),
    ("sample1.erf", 3040, 2024),
    ("sample1.iiq", 9036, 6784),
    ("sample1.kdc", 3072, 2304),
    ("sample1.mos", 5336, 4000),
    ("sample1.mrw", 3016, 2008),
    ("sample1.nef", 4256, 2832),
    ("sample1.nrw", 3648, 2736),
    ("sample1.orf", 3264, 2448),
    ("sample1.pef", 4928, 3264),
    ("sample1.raf", 6240, 4160),
    ("sample1.rw2", 3456, 4608),
    ("sample1.rwl", 2736, 2736),
    ("sample1.srw", 6480, 4320),
    ("sample1.x3f", 4807, 3205),
    ("sample2.cr2", 2592, 1728),
    ("sample2.dng", 5976, 3992),
    ("sample2.x3f", 6208, 4160),
];

#[test]
fn every_sample_format_decodes_to_a_real_image() {
    for &(sample, width, height) in SAMPLES {
        let path = support::sample_path(sample);
        let mut pipeline =
            Pipeline::new_from_file(&path).unwrap_or_else(|e| panic!("{}: {}", sample, e));
        let image = pipeline
            .output_8bit(None)
            .unwrap_or_else(|e| panic!("{}: {}", sample, e));

        assert_eq!((image.width, image.height), (width, height), "{}", sample);

        // a NaN colour matrix, an unshifted CFA or a wrong channel order all show up here
        let pixels = (image.data.len() / 3) as u64;
        let lit = image.data.chunks_exact(3).filter(|p| p.iter().any(|&v| v > 32)).count() as u64;
        assert!(lit * 2 > pixels, "{}: only {} of {} pixels are lit", sample, lit, pixels);

        // Some camera profiles set a white level below the sensor maximum, so a valid
        // but overcast frame can stop short of 255. This still catches black frames.
        let max = image.data.iter().cloned().max().unwrap();
        assert!(max > 150, "{}: brightest pixel is only {}", sample, max);
    }
}
