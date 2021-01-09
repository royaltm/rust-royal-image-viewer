use std::path::Path;
use log::debug;
use image::{io::Reader as ImageReader, GenericImageView, Pixel, ColorType, RgbImage};
use num_traits::cast::ToPrimitive;

use crate::utils::Result;

pub fn load_image_center_into<P: AsRef<Path>>(
    name: P,
    bgpixel: u32,
    buf_width: u32,
    buf_height: u32,
    buf: &mut [u32]
  ) -> Result<()>
{
  let img = load_image(name, buf_width, buf_height)?;
  center_image_into(&img, bgpixel, buf_width, buf_height, buf);
  Ok(())
}

pub fn center_image_into(
    img: &RgbImage,
    bgpixel: u32,
    buf_width: u32,
    buf_height: u32,
    buf: &mut [u32]
  )
{
  let (img_width, img_height) = img.dimensions();

  assert!(img_width <= buf_width && img_height <= buf_height);

  let margin_left = (buf_width - img_width) as usize / 2;
  let margin_right = (buf_width - img_width) as usize - margin_left;
  let margin_top = (buf_height - img_height) / 2;
  let margin_bot = buf_height - img_height - margin_top;

  let yoffset_top = margin_top as usize * buf_width as usize;
  let yoffset_bot = margin_bot as usize * buf_width as usize;

  let mut tgt_iter = buf.iter_mut();

  for tgt in tgt_iter.by_ref().take(yoffset_top) {
    *tgt = bgpixel;
  }

  for pixels in img.rows() {
    for tgt in tgt_iter.by_ref().take(margin_left) {
      *tgt = bgpixel;
    }
    for (src, tgt) in pixels.zip(tgt_iter.by_ref()) {
      if let Some(pixel) = src.into_rgb32() {
        *tgt = pixel;
      }
    }
    for tgt in tgt_iter.by_ref().take(margin_right) {
      *tgt = bgpixel;
    }
  }

  for tgt in tgt_iter.take(yoffset_bot) {
    *tgt = bgpixel;
  }
}

pub fn load_image<P: AsRef<Path>>(
    name: P,
    max_width: u32,
    max_height: u32
  ) -> Result<RgbImage>
{
  let mut img = ImageReader::open(name)?
                .with_guessed_format()?
                .decode()?;

  let (mut img_width, mut img_height) = img.dimensions();

  let color = img.color();
  debug!("color: {:?} bpp:{} alpha:{} color:{} pbits:{} chans:{}", color,
            color.bytes_per_pixel(),
            color.has_alpha(),
            color.has_color(),
            color.bits_per_pixel(),
            color.channel_count());
  debug!("{}x{}", img_width, img_height);

  if img_width > max_width || img_height > max_height {
    let width = img_width.min(max_width);
    let height = img_height.min(max_height);
    let x = if img_width > max_width {
      (img_width - max_width) / 2
    }
    else { 0 };
    let y = if img_height > max_height {
      (img_height - max_height) / 2
    }
    else { 0 };
    img = img.crop_imm(x, y, width, height);
    img_width = img.width();
    img_height = img.height();
    debug!("crop -> {}x{}", img_width, img_height);
  }

  Ok(img.into_rgb8())
}

trait IntoRgb32: Pixel {
  fn into_rgb32(self) -> Option<u32>;
}

impl<P> IntoRgb32 for P where P: Pixel, <P as Pixel>::Subpixel: 'static {
  fn into_rgb32(self) -> Option<u32> {
    let (r, g, b) = match P::COLOR_TYPE {
      ColorType::Bgr8|ColorType::Bgra8|
      ColorType::L8|ColorType::La8|
      ColorType::Rgb8|ColorType::Rgba8 => {
        if let &[r, g, b] = self.to_rgb().channels() {
          (r.to_u8().unwrap(),
           g.to_u8().unwrap(),
           b.to_u8().unwrap())
        }
        else {
          return None
        }
      }
      ColorType::L16|ColorType::La16|
      ColorType::Rgb16|ColorType::Rgba16 => {
        if let &[r, g, b] = self.to_rgb().channels() {
          ((r.to_u16().unwrap() >> 8) as u8,
           (g.to_u16().unwrap() >> 8) as u8,
           (b.to_u16().unwrap() >> 8) as u8)
        }
        else {
          return None
        }
      }
      _ => return None
    };

    Some(from_u8_rgb(r, g, b))
  }
}

#[inline(always)]
pub fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
  u32::from_be_bytes([0, r, g, b])
}
