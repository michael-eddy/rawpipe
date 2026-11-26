use image::{DynamicImage, ImageBuffer};
use num_traits::ops::saturating::Saturating;
use rawpipe::color_conversions::*;
use std::cmp::PartialOrd;
use std::fmt::Debug;

#[test]
fn roundtrip_8bit() {
    for i in 0..=u8::MAX {
        assert_eq!(i, unit_to_u8(u8_to_unit(i)));
    }
}

#[test]
fn roundtrip_16bit() {
    for i in 0..=u16::MAX {
        assert_eq!(i, unit_to_u16(u16_to_unit(i)));
    }
}

#[test]
fn roundtrip_8bit_values_image_crate() {
    // 16x16 pixels with each channel sweeping every u8 value, so the image
    // crate's u8 -> u16 conversion is checked for all 256 inputs
    let mut image_data: Vec<u8> = Vec::with_capacity(16 * 16 * 3);
    for _ in 0..3 {
        for v in 0..=u8::MAX {
            image_data.push(v);
        }
    }
    let image = ImageBuffer::from_raw(16, 16, image_data.clone()).unwrap();
    let out = DynamicImage::ImageRgb8(image).into_rgb16().into_raw();
    assert_eq!(out.len(), image_data.len());

    for (vin, vout) in image_data.into_iter().zip(out.into_iter()) {
        let f = u16_to_unit(vout);
        let out = unit_to_u8(f);
        assert_eq!(out, vin);
    }
}

#[test]
fn roundtrip_16bit_values_image_crate() {
    let mut image_data: Vec<u16> = Vec::with_capacity(256 * 256 * 3);
    for _ in 0..3 {
        for v in 0..=u16::MAX {
            image_data.push(v);
        }
    }
    let image = ImageBuffer::from_raw(256, 256, image_data.clone()).unwrap();
    let out = DynamicImage::ImageRgb16(image).into_rgb16().into_raw();
    assert_eq!(out.len(), image_data.len());

    for (vin, vout) in image_data.into_iter().zip(out.into_iter()) {
        let f = u16_to_unit(vout);
        let out = unit_to_u16(f);
        assert_eq!(out, vin);
    }
}

fn roundtrip_gamma(v: f32) -> f32 {
    let inter = srgb_to_linear(v);
    linear_to_srgb(inter)
}

#[test]
fn roundtrip_8bit_gamma() {
    for i in 0..=u8::MAX {
        assert_eq!(i, unit_to_u8(roundtrip_gamma(u8_to_unit(i))));
    }
}

#[test]
fn roundtrip_16bit_gamma() {
    for i in 0..=u16::MAX {
        assert_eq!(i, unit_to_u16(roundtrip_gamma(u16_to_unit(i))));
    }
}

#[inline(always)]
fn assert_offby<T>(to: (T, T, T), from: (T, T, T), offdown: T, offup: T)
where
    T: Saturating + Debug + PartialOrd + Copy,
{
    let condition = to.0 <= from.0.saturating_add(offup)
        && to.0 >= from.0.saturating_sub(offdown)
        && to.1 <= from.1.saturating_add(offup)
        && to.1 >= from.1.saturating_sub(offdown)
        && to.2 <= from.2.saturating_add(offup)
        && to.2 >= from.2.saturating_sub(offdown);
    if !condition {
        eprintln!("Got {:?} instead of {:?}", to, from);
    }
    assert!(condition)
}

#[test]
fn roundtrip_8bit_lab_xyz() {
    for x in 0..=u8::MAX {
        for y in 0..=u8::MAX {
            for z in 0..=u8::MAX {
                let xf = u8_to_unit(x);
                let yf = u8_to_unit(y);
                let zf = u8_to_unit(z);

                let (l, a, b) = xyz_to_lab(xf, yf, zf);
                let (outxf, outyf, outzf) = lab_to_xyz(l, a, b);

                let outx = unit_to_u8(outxf);
                let outy = unit_to_u8(outyf);
                let outz = unit_to_u8(outzf);

                assert_eq!((outx, outy, outz), (x, y, z));
            }
        }
    }
}

#[test]
fn roundtrip_8bit_lab_rgb() {
    for r in 0..=u8::MAX {
        for g in 0..=u8::MAX {
            for b in 0..=u8::MAX {
                let pixel = [u8_to_unit(r), u8_to_unit(g), u8_to_unit(b), 0.0];
                let multipliers = [1.0, 1.0, 1.0, 1.0];
                let cmatrix = *SRGB_D65_43;
                let rgbmatrix = *XYZ_D65_33;

                let (ll, la, lb) = camera_to_lab(multipliers, cmatrix, &pixel);
                let (outrf, outgf, outbf) = lab_to_rgb(rgbmatrix, &[ll, la, lb]);

                let outr = unit_to_u8(outrf);
                let outg = unit_to_u8(outgf);
                let outb = unit_to_u8(outbf);

                assert_eq!((outr, outg, outb), (r, g, b));
            }
        }
    }
}

#[test]
fn roundtrip_8bit_lab_rgb_gamma() {
    for r in 0..=u8::MAX {
        for g in 0..=u8::MAX {
            for b in 0..=u8::MAX {
                let pixel = [
                    srgb_to_linear(u8_to_unit(r)),
                    srgb_to_linear(u8_to_unit(g)),
                    srgb_to_linear(u8_to_unit(b)),
                    0.0,
                ];
                let multipliers = [1.0, 1.0, 1.0, 1.0];
                let cmatrix = *SRGB_D65_43;
                let rgbmatrix = *XYZ_D65_33;

                let (ll, la, lb) = camera_to_lab(multipliers, cmatrix, &pixel);
                let (outrf, outgf, outbf) = lab_to_rgb(rgbmatrix, &[ll, la, lb]);

                let outrf = linear_to_srgb(outrf);
                let outgf = linear_to_srgb(outgf);
                let outbf = linear_to_srgb(outbf);

                let outr = unit_to_u8(outrf);
                let outg = unit_to_u8(outgf);
                let outb = unit_to_u8(outbf);

                assert_eq!((outr, outg, outb), (r, g, b));
            }
        }
    }
}

#[test]
fn roundtrip_16bit_lab_xyz() {
    for x in (0..=u16::MAX).step_by(89) {
        for y in (0..=u16::MAX).step_by(97) {
            for z in (0..=u16::MAX).step_by(101) {
                let xf = u16_to_unit(x);
                let yf = u16_to_unit(y);
                let zf = u16_to_unit(z);
                let (l, a, b) = xyz_to_lab(xf, yf, zf);
                let (outxf, outyf, outzf) = lab_to_xyz(l, a, b);
                let outx = unit_to_u16(outxf);
                let outy = unit_to_u16(outyf);
                let outz = unit_to_u16(outzf);
                assert_eq!((outx, outy, outz), (x, y, z));
                let x = x >> 8;
                let y = y >> 8;
                let z = z >> 8;
                let outx = unit_to_u8(outxf) as u16;
                let outy = unit_to_u8(outyf) as u16;
                let outz = unit_to_u8(outzf) as u16;

                assert_eq!((outx, outy, outz), (x, y, z));
            }
        }
    }
}

#[test]
fn roundtrip_16bit_lab_rgb() {
    for r in (0..=u16::MAX).step_by(89) {
        for g in (0..=u16::MAX).step_by(97) {
            for b in (0..=u16::MAX).step_by(101) {
                let pixel = [u16_to_unit(r), u16_to_unit(g), u16_to_unit(b), 0.0];
                let multipliers = [1.0, 1.0, 1.0, 1.0];
                let cmatrix = *SRGB_D65_43;
                let rgbmatrix = *XYZ_D65_33;

                let (ll, la, lb) = camera_to_lab(multipliers, cmatrix, &pixel);
                let (outrf, outgf, outbf) = lab_to_rgb(rgbmatrix, &[ll, la, lb]);

                let outr = unit_to_u16(outrf);
                let outg = unit_to_u16(outgf);
                let outb = unit_to_u16(outbf);

                assert_eq!((outr, outg, outb), (r, g, b));

                let r = r >> 8;
                let g = g >> 8;
                let b = b >> 8;

                let outr = unit_to_u8(outrf) as u16;
                let outg = unit_to_u8(outgf) as u16;
                let outb = unit_to_u8(outbf) as u16;

                assert_eq!((outr, outg, outb), (r, g, b));
            }
        }
    }
}

#[test]
fn roundtrip_16bit_lab_rgb_gamma() {
    for r in (0..=u16::MAX).step_by(89) {
        for g in (0..=u16::MAX).step_by(97) {
            for b in (0..=u16::MAX).step_by(101) {
                let pixel = [
                    srgb_to_linear(u16_to_unit(r)),
                    srgb_to_linear(u16_to_unit(g)),
                    srgb_to_linear(u16_to_unit(b)),
                    0.0,
                ];
                let multipliers = [1.0, 1.0, 1.0, 1.0];
                let cmatrix = *SRGB_D65_43;
                let rgbmatrix = *XYZ_D65_33;

                let (ll, la, lb) = camera_to_lab(multipliers, cmatrix, &pixel);
                let ll = roundtrip_gamma(ll);
                let (outrf, outgf, outbf) = lab_to_rgb(rgbmatrix, &[ll, la, lb]);

                let outrf = linear_to_srgb(outrf);
                let outgf = linear_to_srgb(outgf);
                let outbf = linear_to_srgb(outbf);

                let outr = unit_to_u16(outrf) as u16;
                let outg = unit_to_u16(outgf) as u16;
                let outb = unit_to_u16(outbf) as u16;

                assert_offby((outr, outg, outb), (r, g, b), 1, 1);

                let r = r >> 8;
                let g = g >> 8;
                let b = b >> 8;

                let outr = unit_to_u8(outrf) as u16;
                let outg = unit_to_u8(outgf) as u16;
                let outb = unit_to_u8(outbf) as u16;

                assert_eq!((outr, outg, outb), (r, g, b));
            }
        }
    }
}
