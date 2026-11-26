use rawpipe::{scale_down_srgb16, SrgbImage16};

#[test]
fn scaling_noop() {
  let width = 150;
  let height = 150;
  let mut data = vec![0 as u16; width*height*3];
  for (i, o) in data.chunks_exact_mut(1).enumerate() {
    o[0] = i as u16;
  }
  let orig = SrgbImage16 {
    width,
    height,
    data,
  };
  let new = scale_down_srgb16(&orig, width, height);
  assert_eq!(orig, new);
}
