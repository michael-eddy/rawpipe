use rawpipe::SplineFunc;

#[test]
fn extremes() {
  let spline = SplineFunc::new(&[]);
  assert_eq!(spline.interpolate(0.0), 0.0);
  assert_eq!(spline.interpolate(1.0), 1.0);
}

#[test]
fn saturates() {
  let spline = SplineFunc::new(&[]);
  assert_eq!(spline.interpolate(1.5), 1.0);
  assert_eq!(spline.interpolate(-0.2), 0.0);
}

#[test]
fn high_blackpoint() {
  let spline = SplineFunc::new(&[(0.0,0.2)]);
  assert_eq!(spline.interpolate(0.0), 0.2);
}

#[test]
fn low_whitepoint() {
  let spline = SplineFunc::new(&[(1.0,0.8)]);
  assert_eq!(spline.interpolate(1.0), 0.8);
}
