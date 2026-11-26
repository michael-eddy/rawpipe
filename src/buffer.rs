use rayon::prelude::*;

/// Interleaved f32 image data with a known channel count.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PixelBuffer {
  pub width: usize,
  pub height: usize,
  pub channels: usize,
  pub monochrome: bool,
  pub data: Vec<f32>,
}

impl PixelBuffer {
  pub fn new(width: usize, height: usize, channels: usize, monochrome: bool) -> PixelBuffer {
    PixelBuffer {
      width,
      height,
      channels,
      monochrome,
      data: vec![0.0; width*height*channels],
    }
  }

  pub fn mutate_rows<F>(&mut self, closure: &F)
    where F : Fn(&mut [f32], usize)+Sync {

    self.data.par_chunks_mut(self.width*self.channels).enumerate().for_each(|(row_index, row)| {
      closure(row, row_index);
    });
  }

  pub fn mutate_rows_cloned<F>(&self, closure: &F) -> PixelBuffer
    where F : Fn(&mut [f32], usize)+Sync {

    let mut buf = self.clone();
    buf.data.par_chunks_mut(self.width*self.channels).enumerate().for_each(|(row_index, row)| {
      closure(row, row_index);
    });
    buf
  }

  /// Build a buffer with a different channel count from this one, row by row.
  pub fn convert_rows<F>(&self, channels: usize, closure: &F) -> PixelBuffer
    where F : Fn(&mut [f32], &[f32])+Sync {

    let mut out = PixelBuffer::new(self.width, self.height, channels, self.monochrome);
    out.data.par_chunks_mut(out.width*out.channels).enumerate().for_each(|(row_index, row)| {
      closure(row, &self.data[self.width*self.channels*row_index..]);
    });
    out
  }

  /// Resample the area described by the three corners into a `width` x `height` buffer.
  pub fn resample(&self,
    topleft: (isize, isize),
    topright: (isize, isize),
    bottomleft: (isize, isize),
    width: usize,
    height: usize) -> PixelBuffer {

    let newdata = crate::scaling::resample_buffer(&self.data, self.width, self.height,
      topleft, topright, bottomleft, width, height, self.channels, None);

    Self {
      width,
      height,
      channels: self.channels,
      monochrome: self.monochrome,
      data: newdata,
    }
  }
}