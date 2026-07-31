fn main() {
    // Only embed resources on Windows targets.
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // ── Generate .ico from raw 256×256 RGBA data ──────────
    let raw = include_bytes!("res/icon_raw");
    assert_eq!(
        raw.len(),
        256 * 256 * 4,
        "icon_raw must be exactly 256×256 RGBA (262144 bytes)"
    );
    let ico_path = out_dir.join("icon.ico");
    let ico_data = rgba_to_ico(raw, 256, 256);
    std::fs::write(&ico_path, &ico_data).expect("failed to write icon.ico");

    // ── Build Windows resource via winresource ────────────
    let mut res = winresource::WindowsResource::new();

    // Icon
    res.set_icon(ico_path.to_str().unwrap());

    // Version info — FileVersion / ProductVersion are read from
    // Cargo.toml `package.version` automatically.  We only need
    // to set the string fields that Cargo does not provide.
    res.set("CompanyName", "Mikachu2333");
    res.set("FileDescription", "Auto Tip Clock - Desktop Reminder");
    res.set("InternalName", "tip_clock");
    res.set("OriginalFilename", "tip_clock.exe");
    res.set("ProductName", "Tip Clock");
    res.set("LegalCopyright", "Copyright (c) 2026 Mikachu2333");

    // Language: 0x0409 = English (US), 1200 = CP_UTF16
    res.set_language(0x0409);

    res.compile().expect("failed to compile Windows resource");
}

/// Convert raw RGBA pixel data to a BMP-based ICO file.
///
/// The output ICO contains a single 32-bit BGRA image entry
/// with an AND mask (all zeros — every pixel is opaque).
fn rgba_to_ico(rgba: &[u8], w: u32, h: u32) -> Vec<u8> {
    let num_pixels = (w * h) as usize;
    // RGBA → BGRA, flipping vertically (BMP is bottom-up)
    let mut bgra = vec![0u8; num_pixels * 4];
    for y in 0..h {
        let src_row = (y * w) as usize * 4;
        let dst_row = ((h - 1 - y) * w) as usize * 4;
        for x in 0..w as usize {
            let s = src_row + x * 4;
            let d = dst_row + x * 4;
            bgra[d] = rgba[s + 2]; // B
            bgra[d + 1] = rgba[s + 1]; // G
            bgra[d + 2] = rgba[s]; // R
            bgra[d + 3] = rgba[s + 3]; // A
        }
    }

    // AND mask — one bit per pixel, padded to 4-byte boundary per row.
    // All zeros = no pixels are transparent in the mask sense.
    let mask_row_bytes = (w as usize).div_ceil(32) * 4;
    let mask = vec![0u8; mask_row_bytes * h as usize];

    let bmp_data_size = 40 + bgra.len() + mask.len();
    let ico_entry_offset: u32 = 6 + 16; // header + first entry

    let mut out = Vec::with_capacity(ico_entry_offset as usize + bmp_data_size);

    // ── ICO header (6 bytes) ──────────────────────────────
    out.extend_from_slice(&0u16.to_le_bytes()); // reserved
    out.extend_from_slice(&1u16.to_le_bytes()); // type = ICO
    out.extend_from_slice(&1u16.to_le_bytes()); // image count

    // ── ICO directory entry (16 bytes) ────────────────────
    let entry_w = if w >= 256 { 0u8 } else { w as u8 };
    let entry_h = if h >= 256 { 0u8 } else { h as u8 };
    out.push(entry_w);
    out.push(entry_h);
    out.push(0u8); // colour palette size
    out.push(0u8); // reserved
    out.extend_from_slice(&1u16.to_le_bytes()); // planes (always 1 for ICO)
    out.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
    out.extend_from_slice(&(bmp_data_size as u32).to_le_bytes()); // image size
    out.extend_from_slice(&ico_entry_offset.to_le_bytes()); // offset to image data

    // ── BITMAPINFOHEADER (40 bytes) ───────────────────────
    out.extend_from_slice(&40u32.to_le_bytes()); // biSize
    out.extend_from_slice(&(w as i32).to_le_bytes()); // biWidth
    out.extend_from_slice(&((h * 2) as i32).to_le_bytes()); // biHeight (doubled for AND mask)
    out.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    out.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    out.extend_from_slice(&0u32.to_le_bytes()); // biCompression = BI_RGB
    out.extend_from_slice(&0u32.to_le_bytes()); // biSizeImage (0 is fine for BI_RGB)
    out.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
    out.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
    out.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
    out.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant

    // ── Pixel data + AND mask ─────────────────────────────
    out.extend_from_slice(&bgra);
    out.extend_from_slice(&mask);

    out
}
