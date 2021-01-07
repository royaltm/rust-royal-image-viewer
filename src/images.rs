use std::path::Path;
use log::debug;
use image::{io::Reader as ImageReader, GenericImageView, Pixel, ColorType};
use num_traits::cast::ToPrimitive;
use env_logger::Env;

use crate::utils::Result;

pub fn load_image_center_into<P: AsRef<Path>>(
    name: P,
    buf_width: u32,
    buf_height: u32,
    buf: &mut [u32]
  ) -> Result<()>
{
  env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

  let mut img = ImageReader::open(name)?
                .with_guessed_format()?
                .decode()?;

  let (mut img_width, mut img_height) = img.dimensions();

  let color = img.color();
  debug!("{:?}", color);
  debug!("bp:{} a:{} c:{} bits:{} cc:{}", color.bytes_per_pixel(),
            color.has_alpha(),
            color.has_color(),
            color.bits_per_pixel(),
            color.channel_count());
  debug!("{}x{}", img_width, img_height);

  if img_width > buf_width || img_height > buf_height {
    let width = img_width.min(buf_width);
    let height = img_height.min(buf_height);
    let x = if img_width > buf_width {
      (img_width - buf_width) / 2
    }
    else { 0 };
    let y = if img_height > buf_height {
      (img_height - buf_height) / 2
    }
    else { 0 };
    img = img.crop_imm(x, y, width, height);
    img_width = img.width();
    img_height = img.height();
    debug!(" -> {}x{}", img_width, img_height);
  }

  let x0 = (buf_width - img_width) / 2;
  let y0 = (buf_height - img_height) / 2;

  let yoffset = y0 as usize * buf_width as usize;

  for (y, pixels) in img.into_rgb8().rows().enumerate() {
    let bstart = yoffset + y * buf_width as usize + x0 as usize;
    let tline = &mut buf[bstart..bstart + img_width as usize];
    for (src, tgt) in pixels.zip(tline.iter_mut()) {
      if let Some(pixel) = src.into_rgb32() {
        *tgt = pixel;
      }
    }
  }

  Ok(())
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

#[inline]
pub fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}
