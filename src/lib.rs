mod buffer;
pub use self::buffer::PixelBuffer;
mod hasher;

pub mod colorspaces;
pub mod curves;
pub mod demosaic;
pub mod gamma;
pub mod orientation;
pub mod rotatecrop;
pub mod to_float;
pub use self::curves::SplineFunc;
pub use self::orientation::Rotation;

mod opbasics;
mod pipeline;
pub use self::pipeline::*;

pub mod color_conversions;
mod scaling;
#[doc(hidden)]
pub use self::scaling::scale_down_srgb16;

use image::DynamicImage;
use rawler::decoders::RawDecodeParams;
use rawler::get_decoder;
use rawler::rawsource::RawSource;
use std::path::Path;

pub fn decode_8bit<P: AsRef<Path>>(path: P, max_width: usize, max_height: usize) -> Result<SrgbImage, String> {
  let mut pipeline = Pipeline::new_from_file(&path)?;
  pipeline.context.settings.max_width = max_width;
  pipeline.context.settings.max_height = max_height;
  pipeline.output_8bit(None)
}

/// The embedded preview of a raw file, `None` if the file has none.
pub fn preview<P: AsRef<Path>>(path: P) -> Result<Option<DynamicImage>, String> {
  let rawfile = RawSource::new(path.as_ref()).map_err(|err| err.to_string())?;
  let decoder = get_decoder(&rawfile).map_err(|err| err.to_string())?;
  decoder.preview_image(&rawfile, &RawDecodeParams::default()).map_err(|err| err.to_string())
}

/// The embedded thumbnail of a raw file, `None` if the file has none.
pub fn thumbnail<P: AsRef<Path>>(path: P) -> Result<Option<DynamicImage>, String> {
  let rawfile = RawSource::new(path.as_ref()).map_err(|err| err.to_string())?;
  let decoder = get_decoder(&rawfile).map_err(|err| err.to_string())?;
  decoder.thumbnail_image(&rawfile, &RawDecodeParams::default()).map_err(|err| err.to_string())
}
