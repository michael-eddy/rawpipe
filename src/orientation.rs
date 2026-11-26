use crate::opbasics::*;
use std::mem;
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Rotation {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
}

/// Applies the EXIF orientation (rotation and/or mirroring).
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct OpOrientation {
    pub rotation: Rotation,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

impl OpOrientation {
    pub fn new(source: &ImageSource) -> OpOrientation {
        match source {
            ImageSource::Raw(img) => {
                let (rotation, flip_horizontal, flip_vertical) = match img.orientation {
                    None | Some(Orientation::Normal) => (Rotation::Normal, false, false),
                    Some(Orientation::VerticalFlip) => (Rotation::Normal, false, true),
                    Some(Orientation::HorizontalFlip) => (Rotation::Normal, true, false),
                    Some(Orientation::Rotate180) => (Rotation::Rotate180, false, false),
                    Some(Orientation::Transpose) => (Rotation::Rotate90, false, true),
                    Some(Orientation::Rotate90) => (Rotation::Rotate90, false, false),
                    Some(Orientation::Rotate270) => (Rotation::Rotate270, false, false),
                    Some(Orientation::Transverse) => (Rotation::Rotate270, true, false),
                };

                OpOrientation {
                    rotation,
                    flip_horizontal,
                    flip_vertical,
                }
            }
            ImageSource::Raster(_) => OpOrientation {
                rotation: Rotation::Normal,
                flip_horizontal: false,
                flip_vertical: false,
            },
        }
    }
}

impl<'a> ImageOp<'a> for OpOrientation {
    fn name(&self) -> &str {
        "orientation"
    }
    fn run(&self, _context: &PipelineContext, buf: Arc<PixelBuffer>) -> Arc<PixelBuffer> {
        let (f1, f2, f3) = match self.rotation {
            Rotation::Normal => Orientation::Normal,
            Rotation::Rotate90 => Orientation::Rotate90,
            Rotation::Rotate180 => Orientation::Rotate180,
            Rotation::Rotate270 => Orientation::Rotate270,
        }
        .to_flips();

        let orientation = Orientation::from_flips((f1, f2 ^ self.flip_horizontal, f3 ^ self.flip_vertical));

        if orientation == Orientation::Normal {
            buf
        } else {
            Arc::new(rotate_buffer(&buf, &orientation))
        }
    }

    fn output_size(&mut self, width: usize, height: usize) -> (usize, usize) {
        match self.rotation {
            Rotation::Rotate90 | Rotation::Rotate270 => (height, width),
            _ => (width, height),
        }
    }

    fn input_size(&mut self, width: usize, height: usize) -> (usize, usize) {
        self.output_size(width, height)
    }
}

/// Rewrites a three-channel buffer according to an EXIF orientation.
#[doc(hidden)]
pub fn rotate_buffer(buf: &PixelBuffer, orientation: &Orientation) -> PixelBuffer {
    assert_eq!(buf.channels, 3);

    if *orientation == Orientation::Normal {
        return buf.clone();
    }

    if buf.data.len() >= usize::MAX / 2 {
        panic!("Buffer is too wide or high to rotate");
    }

    let mut width = buf.width as isize;
    let mut height = buf.height as isize;

    let (transpose, flip_x, flip_y) = orientation.to_flips();

    let mut base_offset: isize = 0;
    let mut x_step: isize = 3;
    let mut y_step: isize = width * 3;

    if flip_x {
        x_step = -x_step;
        base_offset += (width - 1) * 3;
    }

    if flip_y {
        y_step = -y_step;
        base_offset += width * (height - 1) * 3;
    }

    let mut out = if transpose {
        mem::swap(&mut width, &mut height);
        mem::swap(&mut x_step, &mut y_step);
        PixelBuffer::new(buf.height, buf.width, 3 as usize, buf.monochrome)
    } else {
        PixelBuffer::new(buf.width, buf.height, 3 as usize, buf.monochrome)
    };

    out.mutate_rows(
        &(|line: &mut [f32], row| {
            let line_offset = base_offset + y_step * row as isize;
            for col in 0..width {
                let offset = line_offset + x_step * col;
                for c in 0..3 {
                    line[(col * 3 + c) as usize] = buf.data[(offset + c) as usize];
                }
            }
        }),
    );

    out
}
