use crate::opbasics::*;
use crate::color_conversions::*;
use rawler::imgop::matrix::pseudo_inverse;
use serde::{Serialize, Deserialize};

/// Build camera -> XYZ matrices from the as-shot camera profile's best colour matrix.
fn matrices_from_camera_profile(img: &RawImage) -> Option<([[f32;4];3], [[f32;4];3], [[f32;3];4])> {
  let (_, matrix) = img.best_color_matrix()?;
  if matrix.len() % 3 != 0 {
    return None;
  }
  let components = matrix.len() / 3;
  if !(3..=4).contains(&components) {
    return None;
  }

  let mut xyz_to_cam = [[0.0f32; 3]; 4];
  for row in 0..components {
    for col in 0..3 {
      xyz_to_cam[row][col] = matrix[row * 3 + col];
    }
  }

  // Normalized so that a neutral camera value maps to neutral XYZ
  let mut normalized = xyz_to_cam;
  for row in normalized.iter_mut() {
    let sum: f32 = row.iter().sum();
    for val in row.iter_mut() {
      *val = if sum == 0.0 { 0.0 } else { *val / sum };
    }
  }

  Some((pseudo_inverse(xyz_to_cam), pseudo_inverse(normalized), xyz_to_cam))
}

/// The camera has no usable colour profile.
pub(crate) fn missing_color_matrix(img: &RawImage) -> bool {
  matrices_from_camera_profile(img).is_none()
}

/// White balance gains that render the profile's neutral colour as neutral.
fn neutral_white_balance(xyz_to_cam: [[f32;3];4]) -> [f32;4] {
  const SRGB_TO_XYZ: [[f32;3];3] = [
    [0.412453, 0.357580, 0.180423],
    [0.212671, 0.715160, 0.072169],
    [0.019334, 0.119193, 0.950227],
  ];

  let mut gains = [1.0f32; 4];
  for (i, gain) in gains.iter_mut().enumerate() {
    let mut sum = 0.0;
    for j in 0..3 {
      for k in 0..3 {
        sum += xyz_to_cam[i][k] * SRGB_TO_XYZ[k][j];
      }
    }
    if sum.is_normal() {
      *gain = 1.0 / sum;
    }
  }
  let reference = gains[1];
  [gains[0] / reference, 1.0, gains[2] / reference, gains[3] / reference]
}

/// Single-channel raw data without a colour filter array: a monochrome sensor.
pub(crate) fn is_monochrome(img: &RawImage) -> bool {
  img.cpp == 1 && !img.camera.cfa.is_valid()
}

/// Applies white balance, then the camera → XYZ → Lab transform.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct OpToLab {
  pub cam_to_xyz: [[f32;4];3],
  pub cam_to_xyz_normalized: [[f32;4];3],
  pub xyz_to_cam: [[f32;3];4],
  pub wb_coeffs: [f32;4],
}

fn normalize_white_balance(vals: [f32;4]) -> [f32;4] {
  // Set the green multiplier to 1.0
  let unity = vals[1];
  let norm = |value: f32| if value.is_normal() { value / unity } else { 1.0 };
  [norm(vals[0]), norm(vals[1]), norm(vals[2]), norm(vals[3])]
}

impl OpToLab {
  pub fn new(source: &ImageSource) -> OpToLab {
    match source {
      ImageSource::Raw(img) => {
        if is_monochrome(img) {
          // A monochrome sensor is treated as linear sRGB with no white balance.
          return OpToLab{
            cam_to_xyz: *SRGB_D65_43,
            cam_to_xyz_normalized: *SRGB_D65_43,
            xyz_to_cam: *XYZ_D65_34,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
          };
        }
        // new_from_source() rejects colour raw images without a profile
        let (cam_to_xyz, cam_to_xyz_normalized, xyz_to_cam) =
          matrices_from_camera_profile(img).expect("missing colour profile");
        let coeffs = if !img.wb_coeffs[0].is_normal() ||
                        !img.wb_coeffs[1].is_normal() ||
                        !img.wb_coeffs[2].is_normal() {
          normalize_white_balance(neutral_white_balance(xyz_to_cam))
        } else {
          normalize_white_balance(img.wb_coeffs)
        };
        OpToLab{
          cam_to_xyz,
          cam_to_xyz_normalized,
          xyz_to_cam,
          wb_coeffs: coeffs,
        }
      },
      ImageSource::Raster(_) => {
        OpToLab{
          cam_to_xyz: *SRGB_D65_43,
          cam_to_xyz_normalized: *SRGB_D65_43,
          xyz_to_cam: *XYZ_D65_34,
          wb_coeffs: [1.0, 1.0, 1.0, 0.0],
        }
      }
    }
  }

  pub fn set_temp(&mut self, temp: f32, tint: f32) {
    let xyz = temp_to_xyz(temp);
    let xyz = [xyz[0], xyz[1]/tint, xyz[2]];
    for i in 0..4 {
      self.wb_coeffs[i] = 0.0;
      for j in 0..3 {
        self.wb_coeffs[i] += self.xyz_to_cam[i][j] * xyz[j];
      }
      self.wb_coeffs[i] = self.wb_coeffs[i].recip();
    }
    self.wb_coeffs = normalize_white_balance(self.wb_coeffs);
  }

  pub fn get_temp(&self) -> (f32, f32) {
    let mut xyz = [0.0; 3];
    for i in 0..3 {
      for j in 0..4 {
        let gain = self.wb_coeffs[j];
        if gain > 0.0 {
          xyz[i] += self.cam_to_xyz[i][j] / gain;
        }
      }
    }
    let (temp, tint) = xyz_to_temp(xyz);
    (temp, tint)
  }
}

impl<'a> ImageOp<'a> for OpToLab {
  fn name(&self) -> &str {"to_lab"}
  fn run(&self, _context: &PipelineContext, buf: Arc<PixelBuffer>) -> Arc<PixelBuffer> {
    let cam_to_xyz = if buf.monochrome {
      // Monochrome means we don't need color conversion so it's as if the camera is itself D65 SRGB
      *SRGB_D65_43
    } else {
      self.cam_to_xyz_normalized
    };

    let wb_coeffs = if buf.monochrome {
      [1.0, 1.0, 1.0, 1.0]
    } else {
      normalize_white_balance(self.wb_coeffs)
    };

    Arc::new(buf.convert_rows(3, &(|outb: &mut [f32], inb: &[f32]| {
      for (pixel, lab) in inb.chunks_exact(4).zip(outb.chunks_exact_mut(3)) {
        let (l,a,b) = camera_to_lab(wb_coeffs, cam_to_xyz, pixel);
        lab[0] = l;
        lab[1] = a;
        lab[2] = b;
      }
    })))
  }
}

/// Converts Lab back into linear sRGB.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct OpFromLab {
}

impl OpFromLab {
  pub fn new(_source: &ImageSource) -> OpFromLab {
    OpFromLab{}
  }
}

impl<'a> ImageOp<'a> for OpFromLab {
  fn name(&self) -> &str {"from_lab"}
  fn run(&self, _context: &PipelineContext, buf: Arc<PixelBuffer>) -> Arc<PixelBuffer> {
    Arc::new(buf.mutate_rows_cloned(&(|line: &mut [f32], _| {
      for pix in line.chunks_exact_mut(3) {
        let (r,g,b) = lab_to_rgb(*XYZ_D65_33, pix);

        pix[0] = r;
        pix[1] = g;
        pix[2] = b;
      }
    })))
  }
}
