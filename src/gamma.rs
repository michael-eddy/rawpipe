use crate::opbasics::*;
use crate::color_conversions::*;
use serde::{Serialize, Deserialize};

/// Encodes linear values with the sRGB transfer function (skipped for linear output).
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct OpGamma {
}

impl<'a> OpGamma {
  pub fn new(_source: &ImageSource) -> OpGamma {
    OpGamma{}
  }
}

impl<'a> ImageOp<'a> for OpGamma {
  fn name(&self) -> &str {"gamma"}
  fn run(&self, context: &PipelineContext, buf: Arc<PixelBuffer>) -> Arc<PixelBuffer> {
    if context.settings.linear {
      buf
    } else {
      Arc::new(buf.mutate_rows_cloned(&(|line: &mut [f32], _| {
        for pix in line.chunks_exact_mut(1) {
          pix[0] = linear_to_srgb(pix[0].max(0.0).min(1.0));
        }
      })))
    }
  }
}
