use rawler::imgop::{Dim2, Point, Rect};

use crate::opbasics::*;
use serde::{Serialize, Deserialize};

/// Crops the RAW frame and normalises black/white levels into 0..1 f32 samples.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct OpToFloat {
    pub crop_top: usize,
    pub crop_right: usize,
    pub crop_bottom: usize,
    pub crop_left: usize,
    pub is_cfa: bool,
    pub black_levels: [f32; 4],
    pub white_levels: [f32; 4],
}

fn black_levels(img: &RawImage) -> [f32; 4] {
    let area = img
        .active_area
        .or(img.crop_area)
        .unwrap_or_else(|| Rect::new(Point::new(0, 0), Dim2::new(img.width, img.height)));
    let mut levels = [0.0f32; 4];
    if img.cpp >= 3 {
        for sample in 0..img.blacklevel.cpp.min(3) {
            levels[sample] = img.blacklevel.level(0, 0, sample, area);
        }
    } else {
        for row in 0..2 {
            for col in 0..2 {
                levels[row * 2 + col] = img.blacklevel.level(row, col, 0, area);
            }
        }
    }
    levels
}

impl OpToFloat {
    pub fn new(source: &ImageSource) -> OpToFloat {
        match source {
            ImageSource::Raw(img) => {
                // Offsets of the recommended crop, or the whole frame if there is none
                let [crop_left, crop_top, crop_right, crop_bottom] = img
                    .crop_area
                    .as_ref()
                    .and_then(|rect| rect.as_ltrb_offsets(img.width, img.height))
                    .unwrap_or([0, 0, 0, 0]);
                let black_levels = black_levels(img);
                let mut white_levels = img.whitelevel.as_bayer_array();
                if matches!(img.data, RawImageData::Float(_)) {
                    // rawler falls back to 2^BitsPerSample-1 when a float DNG has no
                    // WhiteLevel tag, but float samples are normalised to 0..1.
                    let synthetic_white = ((1u64 << img.bps.min(32)) - 1) as f32;
                    for level in white_levels.iter_mut() {
                        if *level > 1.0 && *level >= synthetic_white {
                            *level = 1.0;
                        }
                    }
                }
                OpToFloat {
                    crop_top,
                    crop_right,
                    crop_bottom,
                    crop_left,
                    is_cfa: img.camera.cfa.is_valid(),
                    black_levels,
                    white_levels,
                }
            }
            ImageSource::Raster(_) => {
                OpToFloat {
                    crop_top: 0,
                    crop_right: 0,
                    crop_bottom: 0,
                    crop_left: 0,
                    is_cfa: false,
                    black_levels: [0.0, 0.0, 0.0, 0.0], // Unused
                    white_levels: [0.0, 0.0, 0.0, 0.0], // Unused
                }
            }
        }
    }
}

impl<'a> ImageOp<'a> for OpToFloat {
    fn name(&self) -> &str {
        "to_float"
    }
    fn run(&self, context: &PipelineContext, _buf: Arc<PixelBuffer>) -> Arc<PixelBuffer> {
        // FIXME: Doing all the transforms with lookup tables instead of f32 calcs
        //        on every pixel is much faster

        match &context.image {
            ImageSource::Raw(img) => self.run_raw(img),
            ImageSource::Raster(img) => self.run_raster(img),
        }
    }

    fn output_size(&mut self, width: usize, height: usize) -> (usize, usize) {
        let (_, _, width, height) = self.crop_bounds(width, height);
        (width, height)
    }

    // No input_size override: sizing is relative to the scaling done at the
    // demosaic step
}

impl OpToFloat {
    fn crop_bounds(&self, width: usize, height: usize) -> (usize, usize, usize, usize) {
        // Calculate x/y/width/height making sure we get at least a 10x10 "image" to not trip up
        // reasonable assumptions in later ops
        let x = cmp::min(self.crop_left, width - 10);
        let y = cmp::min(self.crop_top, height - 10);
        let width = width - cmp::min(self.crop_left + self.crop_right, width - 10);
        let height = height - cmp::min(self.crop_top + self.crop_bottom, height - 10);
        (x, y, width, height)
    }

    fn run_raw(&self, img: &RawImage) -> Arc<PixelBuffer> {
        // Calculate the levels
        let mins = self.black_levels;
        let ranges = self
            .white_levels
            .iter()
            .enumerate()
            .map(|(i, &x)| x - mins[i])
            .collect::<Vec<f32>>();

        let owidth = img.width;
        let oheight = img.height;
        let (x, y, width, height) = self.crop_bounds(owidth, oheight);

        Arc::new(match img.data {
            RawImageData::Integer(ref data) => {
                if img.cpp == 1 && !self.is_cfa {
                    // We're in a monochrome image so turn it into RGB
                    let mut out = PixelBuffer::new(width, height, 4, true);
                    out.mutate_rows(
                        &(|line: &mut [f32], row| {
                            for (o, i) in line
                                .chunks_exact_mut(4)
                                .zip(data[owidth * (row + y) + x..].chunks_exact(1))
                            {
                                let val = ((i[0] as f32 - mins[0]) / ranges[0]).min(1.0);
                                o[0] = val;
                                o[1] = val;
                                o[2] = val;
                                o[3] = 0.0;
                            }
                        }),
                    );
                    out
                } else if img.cpp == 3 {
                    // We're in an RGB image, turn it into four channel
                    let mut out = PixelBuffer::new(width, height, 4, false);
                    out.mutate_rows(
                        &(|line: &mut [f32], row| {
                            for (o, i) in line
                                .chunks_exact_mut(4)
                                .zip(data[(owidth * (row + y) + x) * 3..].chunks_exact(3))
                            {
                                o[0] = ((i[0] as f32 - mins[0]) / ranges[0]).min(1.0);
                                o[1] = ((i[1] as f32 - mins[1]) / ranges[1]).min(1.0);
                                o[2] = ((i[2] as f32 - mins[2]) / ranges[2]).min(1.0);
                                o[3] = 0.0;
                            }
                        }),
                    );
                    out
                } else {
                    let mut out = PixelBuffer::new(width, height, img.cpp, false);
                    out.mutate_rows(
                        &(|line: &mut [f32], row| {
                            for (o, i) in line
                                .chunks_exact_mut(1)
                                .zip(data[owidth * (row + y) + x..].chunks_exact(1))
                            {
                                o[0] = ((i[0] as f32 - mins[0]) / ranges[0]).min(1.0);
                            }
                        }),
                    );
                    out
                }
            }
            RawImageData::Float(ref data) => {
                if img.cpp == 1 && !self.is_cfa {
                    // We're in a monochrome image so turn it into RGB
                    let mut out = PixelBuffer::new(width, height, 4, true);
                    out.mutate_rows(
                        &(|line: &mut [f32], row| {
                            for (o, i) in line
                                .chunks_exact_mut(4)
                                .zip(data[owidth * (row + y) + x..].chunks_exact(1))
                            {
                                let val = ((i[0] as f32 - mins[0]) / ranges[0]).min(1.0);
                                o[0] = val;
                                o[1] = val;
                                o[2] = val;
                                o[3] = 0.0;
                            }
                        }),
                    );
                    out
                } else if img.cpp == 3 {
                    // We're in an RGB image, turn it into four channel
                    let mut out = PixelBuffer::new(width, height, 4, false);
                    out.mutate_rows(
                        &(|line: &mut [f32], row| {
                            for (o, i) in line
                                .chunks_exact_mut(4)
                                .zip(data[(owidth * (row + y) + x) * 3..].chunks_exact(3))
                            {
                                o[0] = ((i[0] as f32 - mins[0]) / ranges[0]).min(1.0);
                                o[1] = ((i[1] as f32 - mins[1]) / ranges[1]).min(1.0);
                                o[2] = ((i[2] as f32 - mins[2]) / ranges[2]).min(1.0);
                                o[3] = 0.0;
                            }
                        }),
                    );
                    out
                } else {
                    let mut out = PixelBuffer::new(width, height, img.cpp, false);
                    out.mutate_rows(
                        &(|line: &mut [f32], row| {
                            for (o, i) in line
                                .chunks_exact_mut(1)
                                .zip(data[owidth * (row + y) + x..].chunks_exact(1))
                            {
                                o[0] = ((i[0] as f32 - mins[0]) / ranges[0]).min(1.0);
                            }
                        }),
                    );
                    out
                }
            }
        })
    }

    fn run_raster(&self, img: &RasterImage) -> Arc<PixelBuffer> {
        let owidth = img.width() as usize;
        let oheight = img.height() as usize;
        let (x, y, width, height) = self.crop_bounds(owidth, oheight);
        let mut out = PixelBuffer::new(width, height, 4, false);
        let bits_per_channel = img.color().bits_per_pixel() / img.color().channel_count() as u16;

        if bits_per_channel == 8 {
            let data = img.to_rgb8().into_raw();
            out.mutate_rows(
                &(|line: &mut [f32], row| {
                    for (o, i) in line
                        .chunks_exact_mut(4)
                        .zip(data[(owidth * (row + y) + x) * 3..].chunks_exact(3))
                    {
                        o[0] = srgb_to_linear(u8_to_unit(i[0]));
                        o[1] = srgb_to_linear(u8_to_unit(i[1]));
                        o[2] = srgb_to_linear(u8_to_unit(i[2]));
                        o[3] = 0.0;
                    }
                }),
            );
        } else {
            let data = img.to_rgb16().into_raw();
            out.mutate_rows(
                &(|line: &mut [f32], row| {
                    for (o, i) in line
                        .chunks_exact_mut(4)
                        .zip(data[(owidth * (row + y) + x) * 3..].chunks_exact(3))
                    {
                        o[0] = u16_to_unit(i[0]);
                        o[1] = u16_to_unit(i[1]);
                        o[2] = u16_to_unit(i[2]);
                        o[3] = 0.0;
                    }
                }),
            );
        }

        Arc::new(out)
    }
}
