use rawpipe::orientation::rotate_buffer;
use rawpipe::PixelBuffer;
use rawler::Orientation;

// Build a small RGB buffer from one string per row, ' ' being black.
fn from_rgb_str_vec(data: Vec<&str>) -> PixelBuffer {
    let width = data.first().expect("Invalid data for rgb helper function").len();
    let height = data.len();
    let channels = 3;

    let mut pixel_data: Vec<f32> = Vec::with_capacity(width * height * channels);
    for row in data {
        for col in row.chars() {
            let (r, g, b) = match col {
                'R' => (1.0, 0.0, 0.0),
                'G' => (0.0, 1.0, 0.0),
                'B' => (0.0, 0.0, 1.0),
                'O' => (1.0, 1.0, 1.0),
                ' ' => (0.0, 0.0, 0.0),
                c => panic!("Invalid color '{}' sent to rgb expected any of 'RGBO '", c),
            };

            pixel_data.push(r);
            pixel_data.push(g);
            pixel_data.push(b);
        }
    }

    PixelBuffer {
        width,
        height,
        channels,
        monochrome: false,
        data: pixel_data,
    }
}

fn reference_image() -> PixelBuffer {
    from_rgb_str_vec(vec![
        "        ", " RRRRRR ", " GG     ", " BBBB   ", " GG     ", " GG     ", "        ",
    ])
}

#[test]
fn rotate_normal() {
    assert_eq!(rotate_buffer(&reference_image(), &Orientation::Normal), reference_image());
}

#[test]
fn rotate_flip_x() {
    let output = from_rgb_str_vec(vec![
        "        ", " RRRRRR ", "     GG ", "   BBBB ", "     GG ", "     GG ", "        ",
    ]);

    assert_eq!(
        rotate_buffer(&reference_image(), &Orientation::HorizontalFlip),
        output
    );
}

#[test]
fn rotate_flip_y() {
    let output = from_rgb_str_vec(vec![
        "        ", " GG     ", " GG     ", " BBBB   ", " GG     ", " RRRRRR ", "        ",
    ]);
    assert_eq!(
        rotate_buffer(&reference_image(), &Orientation::VerticalFlip),
        output
    );
}

#[test]
fn rotate_rotate90_cw() {
    let output = from_rgb_str_vec(vec![
        "       ", " GGBGR ", " GGBGR ", "   B R ", "   B R ", "     R ", "     R ", "       ",
    ]);
    assert_eq!(rotate_buffer(&reference_image(), &Orientation::Rotate90), output);
}

#[test]
fn rotate_rotate270_cw() {
    let output = from_rgb_str_vec(vec![
        "       ", " R     ", " R     ", " R B   ", " R B   ", " RGBGG ", " RGBGG ", "       ",
    ]);
    assert_eq!(rotate_buffer(&reference_image(), &Orientation::Rotate270), output);
}

#[test]
fn rotate_rotate180() {
    let output = from_rgb_str_vec(vec![
        "        ", "     GG ", "     GG ", "   BBBB ", "     GG ", " RRRRRR ", "        ",
    ]);
    assert_eq!(rotate_buffer(&reference_image(), &Orientation::Rotate180), output);
}

#[test]
fn rotate_transpose() {
    let output = from_rgb_str_vec(vec![
        "       ", " RGBGG ", " RGBGG ", " R B   ", " R B   ", " R     ", " R     ", "       ",
    ]);
    assert_eq!(rotate_buffer(&reference_image(), &Orientation::Transpose), output);
}

#[test]
fn rotate_transverse() {
    let output = from_rgb_str_vec(vec![
        "       ", "     R ", "     R ", "   B R ", "   B R ", " GGBGR ", " GGBGR ", "       ",
    ]);
    assert_eq!(rotate_buffer(&reference_image(), &Orientation::Transverse), output);
}
