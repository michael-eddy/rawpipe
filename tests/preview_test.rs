mod support;

fn sample(name: &str) -> std::path::PathBuf {
    support::sample_path(name)
}

fn preview_size(name: &str) -> Option<(u32, u32)> {
    rawpipe::preview(sample(name))
        .unwrap_or_else(|e| panic!("{}: {}", name, e))
        .map(|image| (image.width(), image.height()))
}

fn thumbnail_size(name: &str) -> Option<(u32, u32)> {
    rawpipe::thumbnail(sample(name))
        .unwrap_or_else(|e| panic!("{}: {}", name, e))
        .map(|image| (image.width(), image.height()))
}

#[test]
fn x3f_preview_and_thumbnail() {
    assert_eq!(preview_size("sample1.x3f"), Some((4704, 3136)));
    assert_eq!(thumbnail_size("sample1.x3f"), Some((221, 147)));

    assert_eq!(preview_size("sample2.x3f"), Some((6192, 4128)));
    // A single embedded preview is not a thumbnail.
    assert_eq!(thumbnail_size("sample2.x3f"), None);
}

#[test]
fn common_formats_expose_a_preview() {
    let previews = [
        ("sample1.cr2", (8688, 5792)),
        ("sample1.cr3", (6000, 4000)),
        ("sample1.nef", (4256, 2832)),
        ("sample1.pef", (640, 424)),
        ("sample1.raf", (4416, 2944)),
        ("sample1.rw2", (1920, 1440)),
    ];
    for (name, expected) in previews {
        assert_eq!(preview_size(name), Some(expected), "{}", name);
    }
}

#[test]
fn missing_file_is_an_error() {
    let missing = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("res/does-not-exist.dng");
    assert!(rawpipe::preview(&missing).is_err());
    assert!(rawpipe::thumbnail(&missing).is_err());
}
