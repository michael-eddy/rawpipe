mod support;

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use rawler::Orientation;
use rawpipe::Pipeline;
use support::{assert_file_ops_do_not_panic, sample_path, verify_real_sample, SampleExpectation};

static BAD_FILE_SEQ: AtomicU64 = AtomicU64::new(0);

fn with_temp_bad_file(name: &str, bytes: &[u8], body: impl FnOnce(&std::path::Path)) {
    let dir = std::env::temp_dir().join(format!(
        "rawpipe_extra_bad_{}_{}",
        std::process::id(),
        BAD_FILE_SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, bytes).unwrap();
    body(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ari_alexa_mini() {
    verify_real_sample(&SampleExpectation {
        sample: "sample1.ari",
        raw_dims: (3424, 2202),
        out_dims: (3424, 2202),
        cpp: 1,
        is_float: false,
        is_linear_raw: false,
        cfa: "BGGR",
        black_levels: [0.0; 4],
        white_levels: [4095.0; 4],
        orientation: None,
        preview: None,
        thumbnail: None,
        scale_max: (320, 320),
        scale_out: (320, 205),
        min_range: 100,
    });
}

#[test]
fn crm_canon_eos_r5c() {
    verify_real_sample(&SampleExpectation {
        sample: "sample1.crm",
        raw_dims: (8224, 4336),
        out_dims: (8192, 4320),
        cpp: 1,
        is_float: false,
        is_linear_raw: false,
        cfa: "RGGB",
        black_levels: [0.0; 4],
        white_levels: [65535.0; 4],
        orientation: Some(Orientation::Normal),
        preview: None,
        thumbnail: None,
        scale_max: (320, 320),
        scale_out: (320, 168),
        min_range: 50,
    });
}

#[test]
fn dcs_kodak_dcs520c_and_eos_dcs3() {
    let specs = [
        SampleExpectation {
            sample: "sample1.dcs",
            raw_dims: (1736, 1160),
            out_dims: (1736, 1160),
            cpp: 1,
            is_float: false,
            is_linear_raw: false,
            cfa: "GRBG",
            black_levels: [176.0; 4],
            white_levels: [4095.0; 4],
            orientation: None,
            preview: Some((156, 104)),
            thumbnail: None,
            scale_max: (320, 320),
            scale_out: (320, 213),
            min_range: 150,
        },
        SampleExpectation {
            sample: "sample2.dcs",
            raw_dims: (1280, 1024),
            out_dims: (1268, 1016),
            cpp: 1,
            is_float: false,
            is_linear_raw: false,
            cfa: "GRBG",
            black_levels: [0.0; 4],
            white_levels: [7855.0; 4],
            orientation: None,
            preview: None,
            thumbnail: None,
            scale_max: (320, 320),
            scale_out: (320, 256),
            min_range: 150,
        },
    ];
    for spec in &specs {
        verify_real_sample(spec);
    }
}

#[test]
fn fff_hasselblad_h5d40() {
    verify_real_sample(&SampleExpectation {
        sample: "sample1.fff",
        raw_dims: (7410, 5586),
        out_dims: (7304, 5478),
        cpp: 1,
        is_float: false,
        is_linear_raw: false,
        cfa: "RGGB",
        black_levels: [256.0; 4],
        white_levels: [65535.0; 4],
        orientation: Some(Orientation::Normal),
        preview: Some((1217, 913)),
        thumbnail: None,
        scale_max: (320, 320),
        scale_out: (320, 239),
        min_range: 150,
    });
}

#[test]
fn mef_mamiya_zd() {
    verify_real_sample(&SampleExpectation {
        sample: "sample1.mef",
        raw_dims: (4016, 5344),
        out_dims: (5344, 4016),
        cpp: 1,
        is_float: false,
        is_linear_raw: false,
        cfa: "RGGB",
        black_levels: [0.0; 4],
        white_levels: [4000.0; 4],
        orientation: Some(Orientation::Rotate90),
        preview: None,
        thumbnail: None,
        scale_max: (320, 320),
        scale_out: (320, 240),
        min_range: 150,
    });
}

#[test]
fn ori_olympus_and_om_system() {
    let specs = [
        SampleExpectation {
            sample: "sample1.ori",
            raw_dims: (4640, 3472),
            out_dims: (4608, 3456),
            cpp: 1,
            is_float: false,
            is_linear_raw: false,
            cfa: "RGGB",
            black_levels: [254.0; 4],
            white_levels: [4000.0; 4],
            orientation: Some(Orientation::Normal),
            preview: Some((3200, 2400)),
            thumbnail: Some((160, 120)),
            scale_max: (320, 320),
            scale_out: (320, 240),
            min_range: 150,
        },
        SampleExpectation {
            sample: "sample2.ori",
            raw_dims: (5220, 3912),
            out_dims: (5184, 3888),
            cpp: 1,
            is_float: false,
            is_linear_raw: false,
            cfa: "RGGB",
            black_levels: [255.0, 254.0, 254.0, 255.0],
            white_levels: [4095.0; 4],
            orientation: Some(Orientation::Normal),
            preview: Some((3200, 2400)),
            thumbnail: Some((160, 120)),
            scale_max: (320, 320),
            scale_out: (320, 239),
            min_range: 150,
        },
    ];
    for spec in &specs {
        verify_real_sample(spec);
    }
}

#[test]
fn qtk_apple_quicktake_100_and_150() {
    let specs = [
        SampleExpectation {
            sample: "sample1.qtk",
            raw_dims: (640, 480),
            out_dims: (640, 480),
            cpp: 1,
            is_float: false,
            is_linear_raw: false,
            cfa: "GRBG",
            black_levels: [0.0; 4],
            white_levels: [1023.0; 4],
            orientation: None,
            preview: None,
            thumbnail: None,
            scale_max: (320, 320),
            scale_out: (320, 240),
            min_range: 150,
        },
        SampleExpectation {
            sample: "sample2.qtk",
            raw_dims: (320, 240),
            out_dims: (320, 240),
            cpp: 1,
            is_float: false,
            is_linear_raw: false,
            cfa: "GRBG",
            black_levels: [0.0; 4],
            white_levels: [1023.0; 4],
            orientation: None,
            preview: None,
            thumbnail: None,
            scale_max: (160, 160),
            scale_out: (160, 120),
            min_range: 150,
        },
        SampleExpectation {
            sample: "sample3.qtk",
            raw_dims: (640, 480),
            out_dims: (640, 480),
            cpp: 1,
            is_float: false,
            is_linear_raw: false,
            cfa: "GRBG",
            black_levels: [0.0; 4],
            white_levels: [16383.0; 4],
            orientation: None,
            preview: None,
            thumbnail: None,
            scale_max: (320, 320),
            scale_out: (320, 240),
            min_range: 150,
        },
    ];
    for spec in &specs {
        verify_real_sample(spec);
    }

    // QuickTake 200 uses JPEG in the camera, while a `qktn` header with byte 5 != 0
    // selects QuickTake 200 in rawler's QtkDecoder and has no camera matrix/definition;
    // it must fail cleanly rather than panicking.
    let mut qt200_hdr = fs::read(sample_path("sample3.qtk")).unwrap();
    qt200_hdr[5] = 1;
    with_temp_bad_file("qt200_sig.qtk", &qt200_hdr, |bad| {
        assert_file_ops_do_not_panic(bad, "qt200 signature");
        assert!(Pipeline::new_from_file(bad).is_err());
    });
}

#[test]
fn raw_naked_multiple_sizes() {
    let specs = [
        // 10-bit RGGB (7,710,960 bytes)
        SampleExpectation {
            sample: "sample1.s3is.raw",
            raw_dims: (2888, 2136),
            out_dims: (2888, 2136),
            cpp: 1,
            is_float: false,
            is_linear_raw: false,
            cfa: "RGGB",
            black_levels: [32.0, 31.0, 32.0, 31.0],
            white_levels: [1023.0; 4],
            orientation: None,
            preview: None,
            thumbnail: None,
            scale_max: (320, 320),
            scale_out: (320, 236),
            min_range: 150,
        },
        // 10-bit GBRG (10,341,600 bytes)
        SampleExpectation {
            sample: "sample2.a720is.raw",
            raw_dims: (3336, 2480),
            out_dims: (3336, 2480),
            cpp: 1,
            is_float: false,
            is_linear_raw: false,
            cfa: "GBRG",
            black_levels: [32.0; 4],
            white_levels: [1023.0; 4],
            orientation: None,
            preview: None,
            thumbnail: None,
            scale_max: (320, 320),
            scale_out: (320, 237),
            min_range: 150,
        },
        // 12-bit GBRG (18,763,488 bytes)
        SampleExpectation {
            sample: "sample3.d10.raw",
            raw_dims: (4104, 3048),
            out_dims: (4104, 3048),
            cpp: 1,
            is_float: false,
            is_linear_raw: false,
            cfa: "GBRG",
            black_levels: [128.0; 4],
            white_levels: [4095.0; 4],
            orientation: None,
            preview: None,
            thumbnail: None,
            scale_max: (320, 320),
            scale_out: (320, 237),
            min_range: 150,
        },
        // 12-bit RGGB with fixed blackpoint 511 and whitepoint 4000 (31,342,080 bytes)
        SampleExpectation {
            sample: "sample4.g7x.raw",
            raw_dims: (5632, 3710),
            out_dims: (5632, 3710),
            cpp: 1,
            is_float: false,
            is_linear_raw: false,
            cfa: "RGGB",
            black_levels: [511.0; 4],
            white_levels: [4000.0; 4],
            orientation: None,
            preview: None,
            thumbnail: None,
            scale_max: (320, 320),
            scale_out: (320, 210),
            min_range: 150,
        },
        // 12-bit RGGB with crop_area (31,663,200 bytes)
        SampleExpectation {
            sample: "sample5.sx710hs.raw",
            raw_dims: (5344, 3950),
            out_dims: (5184, 3888),
            cpp: 1,
            is_float: false,
            is_linear_raw: false,
            cfa: "RGGB",
            black_levels: [127.0; 4],
            white_levels: [4095.0; 4],
            orientation: None,
            preview: None,
            thumbnail: None,
            scale_max: (320, 320),
            scale_out: (320, 239),
            min_range: 150,
        },
    ];
    for spec in &specs {
        verify_real_sample(spec);
    }

    // Naked RAW matches solely by exact file size; an off-by-one byte file must
    // return an error cleanly without panicking.
    let mut wrong_size = fs::read(sample_path("sample1.s3is.raw")).unwrap();
    wrong_size.push(0);
    with_temp_bad_file("off_by_one.raw", &wrong_size, |bad| {
        assert_file_ops_do_not_panic(bad, "naked raw off-by-one size");
        assert!(Pipeline::new_from_file(bad).is_err());
    });
}

#[test]
fn arq_sony_pixelshift_linear_raw() {
    verify_real_sample(&SampleExpectation {
        sample: "sample1.arq",
        raw_dims: (8000, 5320),
        out_dims: (7952, 5304),
        cpp: 3,
        is_float: false,
        is_linear_raw: true,
        cfa: "RGGB",
        black_levels: [512.0, 512.0, 512.0, 0.0],
        white_levels: [15360.0; 4],
        orientation: Some(Orientation::Normal),
        preview: Some((1616, 1080)),
        thumbnail: None,
        scale_max: (320, 320),
        scale_out: (320, 213),
        min_range: 150,
    });
}

#[test]
fn sti_sinar_evolution75() {
    verify_real_sample(&SampleExpectation {
        sample: "sample1.sti",
        raw_dims: (4992, 6668),
        out_dims: (4992, 6668),
        cpp: 1,
        is_float: false,
        is_linear_raw: false,
        cfa: "GRBG",
        black_levels: [50.0; 4],
        white_levels: [16383.0; 4],
        orientation: Some(Orientation::Rotate180),
        preview: Some((624, 833)),
        thumbnail: Some((156, 208)),
        scale_max: (320, 320),
        scale_out: (239, 320),
        min_range: 150,
    });
}
