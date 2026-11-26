use crate::{colorspaces, curves, demosaic, gamma, orientation, rotatecrop, to_float};
use crate::opbasics::*;

use image::{RgbImage, DynamicImage};
use multicache::MultiCache;
use serde::{Deserialize, Serialize};
use rawler::decoders::RawDecodeParams;
use rawler::get_decoder;
use rawler::rawsource::RawSource;

use std::fmt::Debug;
use std::sync::Arc;
use std::io::Write;
use std::path::Path;
use std::hash::{Hash, Hasher};
use std::time::Instant;

/// A RawImage processed into a full 8bit sRGB image with levels and gamma
///
/// The data is a Vec<u8> width width*height*3 elements, where each element is a value
/// between 0 and 255 with the intensity of the color channel with gamma applied
#[derive(Debug, Clone, PartialEq)]
pub struct SrgbImage {
  pub width: usize,
  pub height: usize,
  pub data: Vec<u8>,
}

/// A RawImage processed into a full 16bit sRGB image with levels and gamma
///
/// The data is a Vec<u16> width width*height*3 elements, where each element is a value
/// between 0 and 65535 with the intensity of the color channel with gamma applied
#[derive(Debug, Clone, PartialEq)]
pub struct SrgbImage16 {
  pub width: usize,
  pub height: usize,
  pub data: Vec<u16>,
}

pub type PipelineCache = MultiCache<OpHash, PixelBuffer>;
pub type RasterImage = DynamicImage;

/// Pixels entering the pipeline: a RAW frame from rawler, or an already decoded raster.
#[derive(Debug, Clone)]
pub enum ImageSource {
  Raw(RawImage),
  Raster(RasterImage),
}

impl ImageSource {
  fn width(&self) -> usize {
    match self {
      Self::Raw(raw) => raw.width,
      Self::Raster(img) => img.width() as usize,
    }
  }

  fn height(&self) -> usize {
    match self {
      Self::Raw(raw) => raw.height,
      Self::Raster(img) => img.height() as usize,
    }
  }
}

fn timed<T>(name: &str, body: impl FnOnce() -> T) -> T {
  let from_time = Instant::now();
  let ret = body();
  log::info!("timing: {:>7} ms for |{}", from_time.elapsed().as_millis(), name);
  ret
}

pub trait ImageOp<'a>: Debug+Serialize+Deserialize<'a> {
  fn name(&self) -> &str;
  fn run(&self, context: &PipelineContext, input: Arc<PixelBuffer>) -> Arc<PixelBuffer>;
  fn settings_yaml(&self) -> String {
    serde_yaml::to_string(self).unwrap()
  }
  fn hash(&self, hasher: &mut OpHasher) {
    // Hash the name first as a zero sized struct doesn't actually do any hashing
    hasher.write(self.name().as_bytes()).unwrap();
    hasher.hash_serialized(self);
  }
  /// Given the size of its input, what size of image does this op produce?
  fn output_size(&mut self, width: usize, height: usize) -> (usize, usize) {
    (width, height)
  }
  /// Given the size of its output, what size of input does this op need?
  fn input_size(&mut self, width: usize, height: usize) -> (usize, usize) {
    (width, height)
  }
  /// Clear any state remembered from an earlier run so the pipeline can run again.
  fn reset(&mut self) {}
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct PipelineSettings {
  pub max_width: usize,
  pub max_height: usize,
  pub demosaic_width: usize,
  pub demosaic_height: usize,
  pub linear: bool,
  pub use_fast_path: bool,
}

impl Default for PipelineSettings {
  fn default() -> Self {
    Self {
      max_width: 0,
      max_height: 0,
      demosaic_width: 0,
      demosaic_height: 0,
      linear: false,
      use_fast_path: true,
    }
  }
}

impl PipelineSettings{
  fn hash(&self, hasher: &mut OpHasher) {
    hasher.hash_serialized(self);
  }
}

#[derive(Debug)]
pub struct PipelineContext {
  pub image: ImageSource,
  pub settings: PipelineSettings,
}

impl PipelineContext {
  pub fn mock(width: u32, height: u32) -> Self {
    Self {
      image: ImageSource::Raster(DynamicImage::ImageRgb8(RgbImage::new(width, height))),
      settings: PipelineSettings::default(),
    }
  }
}

/// The ordered set of ops every pipeline run applies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOps {
  pub to_float: to_float::OpToFloat,
  pub demosaic: demosaic::OpDemosaic,
  pub rotatecrop: rotatecrop::OpRotateCrop,
  pub to_lab: colorspaces::OpToLab,
  pub base_curve: curves::OpBaseCurve,
  pub from_lab: colorspaces::OpFromLab,
  pub gamma: gamma::OpGamma,
  pub orientation: orientation::OpOrientation,
}

impl PipelineOps {
  fn new(source: &ImageSource) -> Self {
    let to_float = to_float::OpToFloat::new(source);
    let mut demosaic = demosaic::OpDemosaic::new(source);
    if to_float.is_cfa {
      demosaic.shift_phase(to_float.crop_left % 2, to_float.crop_top % 2);
    }
    Self {
      to_float,
      demosaic,
      rotatecrop: rotatecrop::OpRotateCrop::new(source),
      to_lab: colorspaces::OpToLab::new(source),
      base_curve: curves::OpBaseCurve::new(source),
      from_lab: colorspaces::OpFromLab::new(source),
      gamma: gamma::OpGamma::new(source),
      orientation: orientation::OpOrientation::new(source),
    }
  }
}

impl PartialEq for PipelineOps {
  fn eq(&self, other: &Self) -> bool {
    let mut selfhasher = OpHasher::new();
    selfhasher.hash_serialized(self);
    let mut otherhasher = OpHasher::new();
    otherhasher.hash_serialized(other);
    selfhasher.finish() == otherhasher.finish()
  }
}
impl Eq for PipelineOps {}
impl Hash for PipelineOps {
  fn hash<H: Hasher>(&self, state: &mut H) {
    let mut selfhasher = OpHasher::new();
    selfhasher.hash_serialized(self);
    selfhasher.finish().hash(state);
  }
}

fn reset_ops(ops: &mut PipelineOps) {
  ops.to_float.reset();
  ops.demosaic.reset();
  ops.rotatecrop.reset();
  ops.to_lab.reset();
  ops.base_curve.reset();
  ops.from_lab.reset();
  ops.gamma.reset();
  ops.orientation.reset();
}

fn hash_op<O: ImageOp<'static>>(op: &O, hasher: &mut OpHasher) -> OpHash {
  op.hash(hasher);
  hasher.finish()
}

fn run_op<O: ImageOp<'static>>(
  op: &O,
  index: usize,
  startpos: usize,
  context: &PipelineContext,
  bufin: Arc<PixelBuffer>,
  cache: Option<&PipelineCache>,
  hashes: &[OpHash],
) -> Arc<PixelBuffer> {
  if index < startpos {
    return bufin;
  }
  let label = "    ".to_string() + op.name();
  let out = timed(&label, || op.run(context, bufin.clone()));
  if let Some(cache) = cache {
    cache.put_arc(hashes[index], out.clone(), out.width * out.height * out.channels * 4);
  }
  out
}

/// A configured RAW development pipeline: context + ordered ops.
#[derive(Debug)]
pub struct Pipeline {
  pub context: PipelineContext,
  pub ops: PipelineOps,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineHeader {
  pub version: u32,
  pub filehash: String,
}

impl Pipeline {
  pub fn new_cache(size: usize) -> PipelineCache {
    MultiCache::new(size)
  }

  pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Pipeline, String> {
    let rawfile = RawSource::new(path.as_ref());
    let mut rawfile = match rawfile {
      Ok(r) => r,
      Err(e) => return Err(e.to_string()),
    };
    match get_decoder(&rawfile) {
      Ok(decoder) => {
        match decoder.raw_image(&mut rawfile, &RawDecodeParams::default(), false) {
          Ok(mut rawimage) => {
            match decoder.raw_metadata(&mut rawfile, &RawDecodeParams::default()) {
              Ok(metadata) => {
                if let Some(orientation) = metadata.exif.orientation.and_then(Orientation::from_u16) {
                  rawimage.orientation = Some(orientation);
                }
              },
              Err(e) => return Err(e.to_string()),
            }

            Self::new_from_source(ImageSource::Raw(rawimage))
          },
          Err(e) => Err(e.to_string()),
        }
      },
      Err(e) => Err(e.to_string()),
    }
  }


  pub fn new_from_source(source: ImageSource) -> Result<Pipeline, String> {
    if let ImageSource::Raw(raw) = &source {
      if !colorspaces::is_monochrome(raw) && colorspaces::missing_color_matrix(raw) {
        return Err(format!(
          "rawpipe: no colour matrix for camera '{} {}'",
          raw.make.trim(),
          raw.model.trim()
        ));
      }
    }
    let ops = PipelineOps::new(&source);

    Ok(Pipeline {
      context: PipelineContext {
        image: source,
        settings: PipelineSettings::default(),
      },
      ops,
    })
  }

  pub fn uses_default_ops(&self) -> bool {
    self.ops == PipelineOps::new(&self.context.image)
  }

  pub fn to_yaml(&self) -> String {
    let document = (PipelineHeader {
      version: 0,
      filehash: "0".to_string(),
    }, &self.ops);

    serde_yaml::to_string(&document).unwrap()
  }

  pub fn from_yaml(source: ImageSource, yaml: String) -> Pipeline {
    let (_, ops): (PipelineHeader, PipelineOps) = serde_yaml::from_str(&yaml).unwrap();

    Pipeline {
      context: PipelineContext {
        image: source,
        settings: PipelineSettings::default(),
      },
      ops,
    }
  }

  pub fn run(&mut self, cache: Option<&PipelineCache>) -> Arc<PixelBuffer> {
    timed("  total pipeline", || {
      reset_ops(&mut self.ops);

      // Calculate what size of image we should scale down to at the demosaic stage
      let mut width = self.context.image.width();
      let mut height = self.context.image.height();
      let (w, h) = self.ops.to_float.output_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.demosaic.output_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.rotatecrop.output_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.to_lab.output_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.base_curve.output_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.from_lab.output_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.gamma.output_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.orientation.output_size(width, height);
      width = w;
      height = h;
      log::debug!("Maximum possible image size is {}x{}", width, height);

      let max_width = self.context.settings.max_width;
      let max_height = self.context.settings.max_height;
      let (mut width, mut height) =
        crate::scaling::scaled_size(width, height, max_width, max_height);
      log::debug!("Final image size is {}x{}", width, height);

      let (w, h) = self.ops.orientation.input_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.gamma.input_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.from_lab.input_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.base_curve.input_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.to_lab.input_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.rotatecrop.input_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.demosaic.input_size(width, height);
      width = w;
      height = h;
      let (w, h) = self.ops.to_float.input_size(width, height);
      width = w;
      height = h;
      log::debug!("Needed image size at demosaic {}x{}", width, height);
      self.context.settings.demosaic_width = width;
      self.context.settings.demosaic_height = height;

      // Generate all the hashes for the operations
      let mut hasher = OpHasher::new();
      // Hash the base settings that are potentially used by all operations
      self.context.settings.hash(&mut hasher);
      let mut hashes = Vec::with_capacity(8);
      hashes.push(hash_op(&self.ops.to_float, &mut hasher));
      hashes.push(hash_op(&self.ops.demosaic, &mut hasher));
      hashes.push(hash_op(&self.ops.rotatecrop, &mut hasher));
      hashes.push(hash_op(&self.ops.to_lab, &mut hasher));
      hashes.push(hash_op(&self.ops.base_curve, &mut hasher));
      hashes.push(hash_op(&self.ops.from_lab, &mut hasher));
      hashes.push(hash_op(&self.ops.gamma, &mut hasher));
      hashes.push(hash_op(&self.ops.orientation, &mut hasher));

      // Start with a dummy buffer as to_float doesn't use it, and reuse the
      // calculated buffer of the last op we have a cache hit for
      let mut bufin = Arc::new(PixelBuffer::default());
      let mut startpos = 0;
      if let Some(cache) = cache {
        for (i, hash) in hashes.iter().enumerate() {
          if let Some(buffer) = cache.get(hash) {
            bufin = buffer;
            startpos = i + 1;
          }
        }
      }

      // Do the operations, starting from the last op we have a cached buffer for
      let context = &self.context;
      bufin = run_op(&self.ops.to_float, 0, startpos, context, bufin, cache, &hashes);
      bufin = run_op(&self.ops.demosaic, 1, startpos, context, bufin, cache, &hashes);
      bufin = run_op(&self.ops.rotatecrop, 2, startpos, context, bufin, cache, &hashes);
      bufin = run_op(&self.ops.to_lab, 3, startpos, context, bufin, cache, &hashes);
      bufin = run_op(&self.ops.base_curve, 4, startpos, context, bufin, cache, &hashes);
      bufin = run_op(&self.ops.from_lab, 5, startpos, context, bufin, cache, &hashes);
      bufin = run_op(&self.ops.gamma, 6, startpos, context, bufin, cache, &hashes);
      bufin = run_op(&self.ops.orientation, 7, startpos, context, bufin, cache, &hashes);
      bufin
    })
  }

  pub fn output_8bit(&mut self, cache: Option<&PipelineCache>) -> Result<SrgbImage, String> {
    // If the image is raster and we haven't changed it yet there's no need to go
    // through the whole pipeline. Just go straight to 8bit using the image
    // crate and resize if needed
    if let ImageSource::Raster(ref image) = self.context.image {
      if self.context.settings.use_fast_path && self.uses_default_ops() {
        return Ok(timed("total output_8bit_fastpath()", || {
        let rgb = image.to_rgb8();
        let (width, height) = (rgb.width() as usize, rgb.height() as usize);
        let out = SrgbImage {
          width,
          height,
          data: rgb.into_raw(),
        };
        let (nwidth, nheight) = crate::scaling::scaled_size(
          out.width, out.height,
          self.context.settings.max_width, self.context.settings.max_height
        );
        if nwidth != out.width || nheight != out.height {
          crate::scaling::scale_down_srgb(&out, nwidth, nheight)
        } else {
          out
        }
        }));
      }
    }

    timed("total output_8bit()", || {
      self.context.settings.linear = false;
      let buffer = self.run(cache);

      let image = timed("  8 bit conversion", || {
        let mut image = vec![0 as u8; buffer.width*buffer.height*3];
        for (o, i) in image.chunks_exact_mut(1).zip(buffer.data.iter()) {
          o[0] = unit_to_u8(*i);
        }
        image
      });

      Ok(SrgbImage {
        width: buffer.width,
        height: buffer.height,
        data: image,
      })
    })
  }

  pub fn output_16bit(&mut self, cache: Option<&PipelineCache>) -> Result<SrgbImage16, String> {
    // If the image is raster and we haven't changed it yet there's no need to go
    // through the whole pipeline. Just go straight to 16bit using the image
    // crate and resize if needed
    if let ImageSource::Raster(ref image) = self.context.image {
      if self.context.settings.use_fast_path && self.uses_default_ops() {
        return Ok(timed("total output_16bit_fastpath()", || {
        let rgb = image.to_rgb16();
        let (width, height) = (rgb.width() as usize, rgb.height() as usize);
        let out = SrgbImage16 {
          width,
          height,
          data: rgb.into_raw(),
        };
        let (nwidth, nheight) = crate::scaling::scaled_size(
          out.width, out.height,
          self.context.settings.max_width, self.context.settings.max_height
        );
        if nwidth != out.width || nheight != out.height {
          crate::scaling::scale_down_srgb16(&out, nwidth, nheight)
        } else {
          out
        }
        }));
      }
    }

    timed("total output_16bit()", || {
      self.context.settings.linear = true;
      let buffer = self.run(cache);

      let image = timed("  8 bit conversion", || {
        let mut image = vec![0 as u16; buffer.width*buffer.height*3];
        for (o, i) in image.chunks_exact_mut(1).zip(buffer.data.iter()) {
          o[0] = unit_to_u16(*i);
        }
        image
      });

      Ok(SrgbImage16 {
        width: buffer.width,
        height: buffer.height,
        data: image,
      })
    })
  }
}
