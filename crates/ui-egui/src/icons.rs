//! Bundled images: each Craft App's icon and the launcher's own.

use egui::ImageSource;

/// The 128 px icon of a Craft App ([`catalog::app_icon_png`](flowapps_launcher_core::catalog::app_icon_png)).
pub fn app(id: &str) -> Option<ImageSource<'static>> {
    let png = flowapps_launcher_core::catalog::app_icon_png(id)?;
    let revision = icon_revision(id);
    Some(ImageSource::Bytes { uri: format!("bytes://app-icons/{id}/{revision}.png").into(), bytes: png.into() })
}

fn icon_revision(id: &str) -> &'static str {
    flowapps_launcher_core::hosted_catalog::metadata(id).map_or("bundled", |m| m.icon_sha256.as_str())
}

/// Brand tile for an app whose bundled icon is a plain white glyph on transparency (Gramflow,
/// Voxflow): (app id, tile colour, glyph colour), from each app's brand folder. The installed card
/// shows the glyph on its brand tile; the not-installed card keeps the same layout in greys.
const BRAND_TILES: [(&str, [u8; 3], [u8; 3]); 2] =
    [("gramflow", [0xeb, 0xf6, 0x6c], [0x2b, 0x2b, 0x2b]), ("voxflow", [0xa6, 0x95, 0xf8], [0x33, 0x2d, 0x51])];

/// Not-installed tile and glyph: mid greys, a contrast of about 6:1 so the card reads as "not
/// installed yet" rather than disabled.
const GREY_TILE: [u8; 3] = [0x4a, 0x4a, 0x52];
const GREY_GLYPH: [u8; 3] = [0xd2, 0xd2, 0xd8];

/// A Craft App's icon: in colour for an installed app, in greys for one that is not. Both are made
/// once per app from the same bundled picture and kept in the context.
pub fn app_image(ctx: &egui::Context, id: &str, in_colour: bool) -> Option<egui::Image<'static>> {
    let tile = BRAND_TILES.iter().find(|t| t.0 == id);
    if in_colour && tile.is_none() {
        return app(id).map(egui::Image::new);
    }
    let key = egui::Id::unique(("app-icon", id, in_colour, icon_revision(id)));
    if let Some(texture) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(key)) {
        return Some(egui::Image::new(&texture));
    }
    let ImageSource::Bytes { bytes, .. } = app(id)? else { return None };
    let picture = image::load_from_memory(bytes.as_ref()).ok()?.to_rgba8();
    let size = [picture.width() as usize, picture.height() as usize];
    let pixels = match (tile, in_colour) {
        (Some(&(_, tile, glyph)), true) => on_tile(&picture, tile, glyph),
        (Some(_), false) => on_tile(&picture, GREY_TILE, GREY_GLYPH),
        (None, _) => {
            let mut pixels = picture.into_raw();
            desaturate(&mut pixels);
            pixels
        }
    };
    let name = format!("app-icon-{id}-{in_colour}");
    let texture = ctx.load_texture(name, egui::ColorImage::from_rgba_unmultiplied(size, &pixels), egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(key, texture.clone()));
    Some(egui::Image::new(&texture))
}

/// Draws the white glyph of `picture`, recoloured, centred on a rounded square of `tile`.
fn on_tile(picture: &image::RgbaImage, tile: [u8; 3], glyph: [u8; 3]) -> Vec<u8> {
    let (w, h) = (picture.width(), picture.height());
    let mut out = vec![0u8; (w * h * 4) as usize];
    let radius = w as f32 * 0.22;
    let half = (w as f32 / 2.0, h as f32 / 2.0);
    // Tile: signed distance to a rounded square, with a one pixel soft edge.
    for y in 0..h {
        for x in 0..w {
            let (dx, dy) = ((x as f32 + 0.5 - half.0).abs() - (half.0 - radius), (y as f32 + 0.5 - half.1).abs() - (half.1 - radius));
            let dist = dx.max(0.0).hypot(dy.max(0.0)) + dx.max(dy).min(0.0) - radius;
            let alpha = (0.5 - dist).clamp(0.0, 1.0);
            let at = ((y * w + x) * 4) as usize;
            out[at..at + 3].copy_from_slice(&tile);
            out[at + 3] = (alpha * 255.0) as u8;
        }
    }
    // Glyph: the picture's opaque area, scaled to 58 % of the tile.
    let bounds = picture.enumerate_pixels().filter(|(_, _, p)| p[3] > 8).fold(None, |b: Option<(u32, u32, u32, u32)>, (x, y, _)| {
        Some(b.map_or((x, y, x, y), |(x0, y0, x1, y1)| (x0.min(x), y0.min(y), x1.max(x), y1.max(y))))
    });
    let Some((x0, y0, x1, y1)) = bounds else { return out };
    let (gw, gh) = (x1 - x0 + 1, y1 - y0 + 1);
    let fit = (w as f32 * 0.58) / gw.max(gh) as f32;
    let (tw, th) = (((gw as f32 * fit).round() as u32).max(1), ((gh as f32 * fit).round() as u32).max(1));
    let cropped = image::imageops::crop_imm(picture, x0, y0, gw, gh).to_image();
    let scaled = image::imageops::resize(&cropped, tw, th, image::imageops::FilterType::Lanczos3);
    let (ox, oy) = ((w - tw) / 2, (h - th) / 2);
    for (x, y, p) in scaled.enumerate_pixels() {
        let a = f32::from(p[3]) / 255.0;
        let at = (((oy + y) * w + ox + x) * 4) as usize;
        for c in 0..3 {
            out[at + c] = (f32::from(glyph[c]) * a + f32::from(out[at + c]) * (1.0 - a)).round() as u8;
        }
    }
    out
}

/// Turns RGBA pixels grey by their luminance, leaving the alpha as it was, and lifts the darkest
/// greys so a dark icon doesn't vanish on the dark window.
fn desaturate(pixels: &mut [u8]) {
    let (pixels, _) = pixels.as_chunks_mut::<4>();
    for p in pixels {
        let lum = 0.299 * f32::from(p[0]) + 0.587 * f32::from(p[1]) + 0.114 * f32::from(p[2]);
        let grey = (0x30 as f32 + lum * (255.0 - 0x30 as f32) / 255.0).round() as u8;
        p[..3].fill(grey);
    }
}

/// The FlowApps symbol mark: white on dark backgrounds, used for the app icon and card logos.
pub fn flowapps_logo(_dark_background: bool) -> ImageSource<'static> {
    egui::include_image!("../../../brand/logo only.svg")
}

/// The FlowApps symbol mark's width / height (its SVG view box: 63.4 × 42.91).
pub const FLOWAPPS_LOGO_ASPECT: f32 = 63.4 / 42.91;

/// A social service's logo (Simple Icons, white, for tinting), by its name in `catalog::SOCIAL`.
pub fn social(service: &str) -> Option<ImageSource<'static>> {
    Some(match service {
        "X" => egui::include_image!("../../../assets/social/x.svg"),
        "YouTube" => egui::include_image!("../../../assets/social/youtube.svg"),
        "TikTok" => egui::include_image!("../../../assets/social/tiktok.svg"),
        "Instagram" => egui::include_image!("../../../assets/social/instagram.svg"),
        "GitHub" => egui::include_image!("../../../assets/social/github.svg"),
        "Discord" => egui::include_image!("../../../assets/social/discord.svg"),
        "Website" => egui::include_image!("../../../assets/social/globe.svg"),
        "LinkedIn" => egui::include_image!("../../../assets/social/linkedin.svg"),
        _ => return None,
    })
}

/// The FlowApps symbol, used in dialog headers.
pub fn artcraft_mark() -> ImageSource<'static> {
    egui::include_image!("../../../brand/logo only.svg")
}

#[cfg(test)]
mod tests {
    use super::desaturate;

    #[test]
    fn grey_pixels_have_no_colour_and_keep_their_alpha() {
        let mut px = [255, 0, 0, 200, 0, 128, 255, 17, 10, 10, 10, 255];
        desaturate(&mut px);
        for p in px.as_chunks::<4>().0 {
            assert!(p[0] == p[1] && p[1] == p[2], "{p:?}");
        }
        assert_eq!([px[3], px[7], px[11]], [200, 17, 255]);
        // Pure red is darker than pure white, and even the darkest grey is lifted off the black window.
        assert!(px[0] < 255 && px[8] >= 0x30);
    }
}

#[cfg(test)]
mod preview_tests {
    #[test]
    #[ignore = "writes preview PNGs to $ICON_PREVIEW_DIR"]
    fn write_previews() {
        let Ok(dir) = std::env::var("ICON_PREVIEW_DIR") else { return };
        let mut sheet = image::RgbaImage::from_pixel(128 * 6, 128, image::Rgba([10, 10, 12, 255]));
        for (n, (id, colour)) in [("gramflow", true), ("gramflow", false), ("voxflow", true), ("voxflow", false), ("mailflow", true), ("mailflow", false)].into_iter().enumerate() {
            let png = flowapps_launcher_core::catalog::app_icon_png(id).unwrap_or_default();
            let picture = image::load_from_memory(png).unwrap().to_rgba8();
            let tile = super::BRAND_TILES.iter().find(|t| t.0 == id);
            let px = match (tile, colour) {
                (Some(&(_, t, g)), true) => super::on_tile(&picture, t, g),
                (Some(_), false) => super::on_tile(&picture, super::GREY_TILE, super::GREY_GLYPH),
                (None, true) => picture.clone().into_raw(),
                (None, false) => {
                    let mut p = picture.clone().into_raw();
                    super::desaturate(&mut p);
                    p
                }
            };
            let icon = image::RgbaImage::from_raw(picture.width(), picture.height(), px).unwrap();
            let icon = image::imageops::resize(&icon, 128, 128, image::imageops::FilterType::Lanczos3);
            image::imageops::overlay(&mut sheet, &icon, (n as i64) * 128, 0);
        }
        sheet.save(format!("{dir}/icons.png")).unwrap();
    }
}
