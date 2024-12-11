use crate::formats::rgb::RGB8Source;
use crate::formats::rgb2yuv::{write_yuv_by_pixel, write_yuv_scalar};
use crate::formats::RGBSource;

/// Allows the [Encoder](crate::encoder::Encoder) to be generic over a YUV source.
pub trait YUVSource {
    /// Size of the image as `(w, h)`.
    #[must_use]
    fn dimensions_i32(&self) -> (i32, i32) {
        let (w, h) = self.dimensions();
        (w as i32, h as i32)
    }

    /// Size of the image as `(w, h)`.
    #[must_use]
    fn dimensions(&self) -> (usize, usize);

    /// YUV strides as `(y, u, v)`.
    ///
    /// For now you should make sure `u == v`.
    #[must_use]
    fn strides(&self) -> (usize, usize, usize);

    /// YUV strides as `(y, u, v)`.
    ///
    /// For now you should make sure `u == v`.
    #[must_use]
    fn strides_i32(&self) -> (i32, i32, i32) {
        let (y, u, v) = self.strides();
        (y as i32, u as i32, v as i32)
    }

    /// Y buffer, should be of size `dimension.1 * strides.0`.
    #[must_use]
    fn y(&self) -> &[u8];

    /// U buffer, should be of size `dimension.1 * strides.1`.
    #[must_use]
    fn u(&self) -> &[u8];

    /// V buffer, should be of size `dimension.1 * strides.2`.
    #[must_use]
    fn v(&self) -> &[u8];

    /// Estimates how many bytes you'll need to store this YUV in an `&[u8]` RGB array.
    ///
    /// This function should return `w * h * 3`.
    #[must_use]
    fn estimate_rgb_u8_size(&self) -> usize {
        let (w, h) = self.dimensions();
        w * h * 3
    }

    /// Estimates how many bytes you'll need to store this YUV in an `&[u8]` RGBA array.
    ///
    /// This function should return `w * h * 4`.
    #[must_use]
    fn estimate_rgba_u8_size(&self) -> usize {
        let (w, h) = self.dimensions();
        w * h * 4
    }
}

/// Converts RGB to YUV data.
#[must_use]
pub struct YUVBuffer {
    yuv: Vec<u8>,
    width: usize,
    height: usize,
}

impl YUVBuffer {
    /// Creates a new YUV buffer from the given vec.
    ///
    /// The vec's length should be `3 * (width * height) / 2`.
    ///
    /// # Panics
    ///
    /// May panic if the given sizes are not multiples of 2, or the yuv buffer's size mismatches.
    pub fn from_vec(yuv: Vec<u8>, width: usize, height: usize) -> Self {
        assert_eq!(width % 2, 0, "width needs to be a multiple of 2");
        assert_eq!(height % 2, 0, "height needs to be a multiple of 2");
        assert_eq!(yuv.len(), (3 * (width * height)) / 2, "YUV buffer needs to be properly sized");

        Self { yuv, width, height }
    }

    /// Allocates a new YUV buffer with the given width and height.
    ///
    /// Both dimensions must be even.
    ///
    /// # Panics
    ///
    /// May panic if the given sizes are not multiples of 2.
    pub fn new(width: usize, height: usize) -> Self {
        assert_eq!(width % 2, 0, "width needs to be a multiple of 2");
        assert_eq!(height % 2, 0, "height needs to be a multiple of 2");

        Self {
            yuv: vec![0u8; (3 * (width * height)) / 2],
            width,
            height,
        }
    }

    /// Allocates a new YUV buffer with the given width and height and data.
    ///
    /// # Panics
    ///
    /// May panic if invoked with an RGB source where the dimensions are not multiples of 2.
    pub fn from_rgb_source(rgb: impl RGBSource) -> Self {
        let mut rval = Self::new(rgb.dimensions().0, rgb.dimensions().1);
        rval.read_rgb(rgb);
        rval
    }

    /// Allocates a new YUV buffer with the given width and height and data.
    ///
    /// This is the faster version of [`Self::from_rgb_source`] and you should generally
    /// use this one.
    ///
    /// # Panics
    ///
    /// May panic if invoked with an RGB source where the dimensions are not multiples of 2.
    pub fn from_rgb8_source(rgb: impl RGB8Source) -> Self {
        let mut rval = Self::new(rgb.dimensions().0, rgb.dimensions().1);
        rval.read_rgb(rgb);
        rval
    }

    /// Reads an RGB buffer, converts it to YUV and stores it.
    ///
    /// # Panics
    ///
    /// May panic if the given `rgb` does not match the internal format.
    pub fn read_rgb(&mut self, rgb: impl RGBSource) {
        let dimensions = self.dimensions();
        let u_base = self.width * self.height;
        let v_base = u_base / 4;
        let (y_buf, uv_buf) = self.yuv.split_at_mut(u_base);
        let (u_buf, v_buf) = uv_buf.split_at_mut(v_base);
        write_yuv_by_pixel(rgb, dimensions, y_buf, u_buf, v_buf);
    }

    /// Reads an RGB8 buffer, converts it to YUV and stores it.
    ///
    /// This is the faster version of [`Self::read_rgb`] and you should generally use this one.
    ///
    /// # Panics
    ///
    /// May panic if the given `rgb` does not match the internal format.
    pub fn read_rgb8(&mut self, rgb: impl RGB8Source) {
        let dimensions = self.dimensions();
        let u_base = self.width * self.height;
        let v_base = u_base / 4;
        let (y_buf, uv_buf) = self.yuv.split_at_mut(u_base);
        let (u_buf, v_buf) = uv_buf.split_at_mut(v_base);
        write_yuv_scalar(rgb, dimensions, y_buf, u_buf, v_buf);
    }
}

impl YUVSource for YUVBuffer {
    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn strides(&self) -> (usize, usize, usize) {
        (self.width, self.width / 2, self.width / 2)
    }

    fn y(&self) -> &[u8] {
        &self.yuv[0..self.width * self.height]
    }

    fn u(&self) -> &[u8] {
        let base_u = self.width * self.height;
        &self.yuv[base_u..base_u + base_u / 4]
    }

    fn v(&self) -> &[u8] {
        let base_u = self.width * self.height;
        let base_v = base_u + base_u / 4;
        &self.yuv[base_v..]
    }
}

/// Convenience wrapper if you already have YUV-sliced data from some other place.
#[must_use]
pub struct YUVSlices<'a> {
    dimensions: (usize, usize),
    yuv: (&'a [u8], &'a [u8], &'a [u8]),
    strides: (usize, usize, usize),
}

impl<'a> YUVSlices<'a> {
    /// Creates a new YUV slice in 4:2:0 format.
    ///
    /// Assume you have some dimension `(w, h)` that is your actual image size. In addition,
    /// you will have strides `(sy, su, sv)` that specify how many pixels / bytes per row
    /// are actually used be used. Strides must be larger or equal than `w` (y) or `w / 2` (uv)
    /// respectively.
    ///
    /// # Panics
    ///
    /// This will panic if the given slices, strides or dimensions don't match.
    pub fn new(yuv: (&'a [u8], &'a [u8], &'a [u8]), dimensions: (usize, usize), strides: (usize, usize, usize)) -> Self {
        assert!(strides.0 >= dimensions.0);
        assert!(strides.1 >= dimensions.0 / 2);
        assert!(strides.2 >= dimensions.0 / 2);

        assert_eq!(dimensions.1 * strides.0, yuv.0.len());
        assert_eq!((dimensions.1 / 2) * strides.1, yuv.1.len());
        assert_eq!((dimensions.1 / 2) * strides.2, yuv.2.len());

        Self {
            dimensions,
            yuv,
            strides,
        }
    }
}

impl YUVSource for YUVSlices<'_> {
    fn dimensions(&self) -> (usize, usize) {
        self.dimensions
    }

    fn strides(&self) -> (usize, usize, usize) {
        self.strides
    }

    fn y(&self) -> &[u8] {
        self.yuv.0
    }

    fn u(&self) -> &[u8] {
        self.yuv.1
    }

    fn v(&self) -> &[u8] {
        self.yuv.2
    }
}

#[cfg(test)]
mod tests {
    use super::{YUVBuffer, YUVSlices};
    use crate::formats::yuv2rgb::{write_rgb8_f32x8, write_rgb8_scalar};
    use crate::formats::{RgbSliceU8, YUVSource};

    #[test]
    fn rgb_to_yuv_conversion_black_2x2() {
        let rgb_source = RgbSliceU8::new(&[0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8], (2, 2));
        let yuv = YUVBuffer::from_rgb_source(rgb_source);
        assert_eq!(yuv.y(), [16u8, 16u8, 16u8, 16u8]);
        assert_eq!(yuv.u(), [128u8]);
        assert_eq!(yuv.v(), [128u8]);
        assert_eq!(yuv.strides_i32().0, 2);
        assert_eq!(yuv.strides_i32().1, 1);
        assert_eq!(yuv.strides_i32().2, 1);
    }

    #[test]
    fn rgb_to_yuv_conversion_white_4x2() {
        let data = &[
            255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
            255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        ];
        let rgb_source = RgbSliceU8::new(data, (4, 2));
        let yuv = YUVBuffer::from_rgb_source(rgb_source);
        assert_eq!(yuv.y(), [235u8, 235u8, 235u8, 235u8, 235u8, 235u8, 235u8, 235u8]);
        assert_eq!(yuv.u(), [128u8, 128u8]);
        assert_eq!(yuv.v(), [128u8, 128u8]);
        assert_eq!(yuv.strides_i32().0, 4);
        assert_eq!(yuv.strides_i32().1, 2);
        assert_eq!(yuv.strides_i32().2, 2);
    }

    #[test]
    fn rgb_to_yuv_conversion_red_2x4() {
        let data = &[
            255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8,
            0u8, 255u8, 0u8, 0u8,
        ];
        let rgb_source = RgbSliceU8::new(data, (4, 2));
        let yuv = YUVBuffer::from_rgb_source(rgb_source);

        assert_eq!(yuv.y(), [81u8, 81u8, 81u8, 81u8, 81u8, 81u8, 81u8, 81u8]);
        assert_eq!(yuv.u(), [90u8, 90u8]);
        assert_eq!(yuv.v(), [239u8, 239u8]);
        assert_eq!(yuv.strides_i32().0, 4);
        assert_eq!(yuv.strides_i32().1, 2);
        assert_eq!(yuv.strides_i32().2, 2);
    }

    #[test]
    #[should_panic]
    fn test_new_stride_less_than_width() {
        let y = vec![0u8; 10];
        let u = vec![0u8; 5];
        let v = vec![0u8; 5];
        let _ = YUVSlices::new((&y, &u, &v), (10, 1), (9, 5, 5));
    }

    #[test]
    #[should_panic]
    fn test_new_u_stride_less_than_half_width() {
        let y = vec![0u8; 20];
        let u = vec![0u8; 5];
        let v = vec![0u8; 5];
        let _ = YUVSlices::new((&y, &u, &v), (10, 2), (10, 4, 5));
    }

    #[test]
    #[should_panic]
    fn test_new_v_stride_less_than_half_width() {
        let y = vec![0u8; 20];
        let u = vec![0u8; 5];
        let v = vec![0u8; 5];
        let _ = YUVSlices::new((&y, &u, &v), (10, 2), (10, 5, 4));
    }

    #[test]
    #[should_panic]
    fn test_new_y_length_not_matching() {
        let y = vec![0u8; 19];
        let u = vec![0u8; 5];
        let v = vec![0u8; 5];
        let _ = YUVSlices::new((&y, &u, &v), (10, 2), (10, 5, 5));
    }

    #[test]
    #[should_panic]
    fn test_new_u_length_not_matching() {
        let y = vec![0u8; 20];
        let u = vec![0u8; 4];
        let v = vec![0u8; 5];
        let _ = YUVSlices::new((&y, &u, &v), (10, 2), (10, 5, 5));
    }

    #[test]
    #[should_panic]
    fn test_new_v_length_not_matching() {
        let y = vec![0u8; 20];
        let u = vec![0u8; 5];
        let v = vec![0u8; 4];
        let _ = YUVSlices::new((&y, &u, &v), (10, 2), (10, 5, 5));
    }

    #[test]
    fn test_new_valid() {
        let y = vec![0u8; 20];
        let u = vec![0u8; 5];
        let v = vec![0u8; 5];
        let _ = YUVSlices::new((&y, &u, &v), (10, 2), (10, 5, 5));
    }

    /// Test every YUV value and see, if the SIMD version delivers a similar RGB value.
    #[test]
    fn test_write_rgb8_f32x8_spectrum() {
        let dim = (8, 2);
        let strides = (8, 4, 4);

        // build artificial YUV planes containing the entire YUV spectrum
        for y in 0..=255u8 {
            for u in 0..=255u8 {
                for v in 0..=255u8 {
                    let (y_plane, u_plane, v_plane) = (vec![y; 16], vec![u; 4], vec![v; 4]);
                    let mut target = vec![0; dim.0 * dim.1 * 3];
                    write_rgb8_scalar(&y_plane, &u_plane, &v_plane, dim, strides, &mut target);

                    let mut target2 = vec![0; dim.0 * dim.1 * 3];
                    write_rgb8_f32x8(&y_plane, &u_plane, &v_plane, dim, strides, &mut target2);

                    // compare first pixel
                    for i in 0..3 {
                        // Due to different CPU architectures the values may slightly change and may not be exactly equal.
                        // allow difference of 1 / 255 (ca. 0.4%)
                        let diff = (target[i] as i32 - target2[i] as i32).abs();
                        assert!(diff <= 1, "YUV: {:?} yielded different results", (y, u, v));
                    }
                }
            }
        }
    }
}
