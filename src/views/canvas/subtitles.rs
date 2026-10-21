use crate::models::types::subtitle::{MaskType, WordAnimation};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::ACCENT_CYAN;
use egui::{Context, Pos2, Rect, Stroke, Vec2};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static FONT_DEFS: Mutex<Option<egui::FontDefinitions>> = Mutex::new(None);
static LOADED_FONTS: Mutex<Option<HashSet<String>>> = Mutex::new(None);
static AVAILABLE_FONTS: Mutex<Option<Vec<String>>> = Mutex::new(None);

pub fn register_font(ctx: &egui::Context, name: &str, bytes: Vec<u8>) {
    let mut lock = FONT_DEFS.lock().unwrap();
    let font_defs = lock.get_or_insert_with(egui::FontDefinitions::default);
    let font_name = name.to_string();

    font_defs
        .font_data
        .insert(font_name.clone(), egui::FontData::from_owned(bytes));
    let family_key = egui::FontFamily::Name(font_name.clone().into());
    font_defs
        .families
        .entry(family_key)
        .or_default()
        .insert(0, font_name.clone());

    font_defs
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push(font_name);

    // NOTE: egui only applies these definitions at the start of the NEXT frame.
    // Until then the new family is NOT usable (using it panics inside epaint).
    ctx.set_fonts(font_defs.clone());
}

pub fn is_font_registered(name: &str) -> bool {
    let lock = FONT_DEFS.lock().unwrap();
    if let Some(defs) = lock.as_ref() {
        let family_key = egui::FontFamily::Name(name.to_string().into());
        defs.families.contains_key(&family_key)
    } else {
        false
    }
}

fn is_font_live(ctx: &egui::Context, family: &egui::FontFamily) -> bool {
    ctx.fonts(|f| f.families().contains(family))
}

fn get_font_candidates(family: &str, bold: bool) -> Vec<String> {
    let lower = family.to_lowercase();
    let mut names = Vec::new();

    let style_suffixes: &[&str] = if bold {
        &["bd", "-bold", "_bold", "b", "-b", "bold"]
    } else {
        &["", "-regular", "_regular", "-normal", "regular", "r"]
    };

    let base_keys: Vec<&str> = match lower.as_str() {
        "arial" => vec!["arial", "liberationsans", "dejavusans", "freesans"],
        "times new roman" | "times" => vec![
            "times",
            "timesnewroman",
            "liberationserif",
            "dejavuserif",
            "freeserif",
        ],
        "courier new" | "courier" => vec![
            "cour",
            "couriernew",
            "courier",
            "liberationmono",
            "dejavusansmono",
        ],
        "comic sans ms" | "comic sans" => vec!["comic", "comicsans", "comicsansms"],
        "trebuchet ms" | "trebuchet" => vec!["trebuc", "trebuchet", "trebuchetms"],
        "verdana" => vec!["verdana"],
        "georgia" => vec!["georgia"],
        "impact" => vec!["impact"],
        "helvetica" => vec!["helvetica", "arial", "liberationsans", "dejavusans"],
        "roboto" => vec!["roboto"],
        "montserrat" => vec!["montserrat"],
        "bebas neue" => vec!["bebasneue", "bebas"],
        "open sans" => vec!["opensans", "open-sans"],
        "futura" => vec!["futura"],
        _ => vec![lower.as_str()],
    };

    for base in &base_keys {
        for s in style_suffixes {
            names.push(format!("{}{}", base, s));
        }
        names.push(base.to_string());
    }
    names
}

fn get_system_font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if let Ok(windir) = std::env::var("WINDIR") {
            dirs.push(PathBuf::from(windir).join("Fonts"));
        } else {
            dirs.push(PathBuf::from("C:\\Windows\\Fonts"));
        }
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            dirs.push(
                PathBuf::from(localappdata)
                    .join("Microsoft")
                    .join("Windows")
                    .join("Fonts"),
            );
        }
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            dirs.push(PathBuf::from(&userprofile).join("Downloads"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from("/System/Library/Fonts"));
        dirs.push(PathBuf::from("/System/Library/Fonts/Supplemental"));
        dirs.push(PathBuf::from("/Library/Fonts"));
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(&home).join("Library/Fonts"));
            dirs.push(PathBuf::from(&home).join("Downloads"));
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        dirs.push(PathBuf::from("/usr/share/fonts"));
        dirs.push(PathBuf::from("/usr/local/share/fonts"));
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(&home);
            dirs.push(home_path.join(".local/share/fonts"));
            dirs.push(home_path.join(".fonts"));
            dirs.push(home_path.join("Downloads"));
        }
    }

    dirs
}

fn read_ttf_family_name(path: &Path) -> Option<String> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    let mut file = File::open(path).ok()?;
    let mut buf = [0u8; 12];
    file.read_exact(&mut buf).ok()?;

    let magic = &buf[0..4];
    if magic != [0x00, 0x01, 0x00, 0x00] && magic != b"OTTO" && magic != b"ttcf" && magic != b"true"
    {
        return None;
    }

    let num_tables = if magic == b"ttcf" {
        let mut ttc_head = [0u8; 16];
        file.seek(SeekFrom::Start(12)).ok()?;
        file.read_exact(&mut ttc_head).ok()?;
        let off = u32::from_be_bytes(ttc_head[0..4].try_into().unwrap());
        file.seek(SeekFrom::Start(off as u64)).ok()?;
        file.read_exact(&mut buf).ok()?;
        u16::from_be_bytes([buf[4], buf[5]])
    } else {
        u16::from_be_bytes([buf[4], buf[5]])
    };

    for _ in 0..num_tables {
        let mut table_dir = [0u8; 16];
        file.read_exact(&mut table_dir).ok()?;
        if &table_dir[0..4] == b"name" {
            let offset = u32::from_be_bytes(table_dir[8..12].try_into().unwrap());
            file.seek(SeekFrom::Start(offset as u64)).ok()?;
            let mut name_head = [0u8; 6];
            file.read_exact(&mut name_head).ok()?;
            let count = u16::from_be_bytes([name_head[2], name_head[3]]);
            let string_offset = u16::from_be_bytes([name_head[4], name_head[5]]);

            let mut name_records = vec![0u8; count as usize * 12];
            file.read_exact(&mut name_records).ok()?;

            for i in 0..count as usize {
                let record = &name_records[i * 12..(i + 1) * 12];
                let platform_id = u16::from_be_bytes([record[0], record[1]]);
                let name_id = u16::from_be_bytes([record[6], record[7]]);
                if name_id == 1 || name_id == 4 {
                    // Family or Full Name
                    let length = u16::from_be_bytes([record[8], record[9]]);
                    let off = u16::from_be_bytes([record[10], record[11]]);

                    let cur_pos = file.seek(SeekFrom::Current(0)).unwrap_or(0);

                    file.seek(SeekFrom::Start(
                        (offset + string_offset as u32 + off as u32) as u64,
                    ))
                    .ok()?;
                    let mut name_bytes = vec![0u8; length as usize];
                    file.read_exact(&mut name_bytes).ok()?;

                    file.seek(SeekFrom::Start(cur_pos)).ok()?;

                    if platform_id == 0 || platform_id == 3 {
                        let mut utf16 = Vec::new();
                        for chunk in name_bytes.chunks_exact(2) {
                            utf16.push(u16::from_be_bytes([chunk[0], chunk[1]]));
                        }
                        if let Ok(s) = String::from_utf16(&utf16) {
                            let clean = s.trim().replace('\0', "");
                            if !clean.is_empty() {
                                return Some(clean);
                            }
                        }
                    } else if platform_id == 1 {
                        let s = String::from_utf8_lossy(&name_bytes).to_string();
                        let clean = s.trim().replace('\0', "");
                        if !clean.is_empty() {
                            return Some(clean);
                        }
                    }
                }
            }
        }
    }
    None
}

fn scan_dir_for_font(dir: &Path, candidates: &[String], depth: usize) -> Option<PathBuf> {
    if depth > 4 || !dir.is_dir() {
        return None;
    }
    let entries = std::fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();

    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() {
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ext_lower == "ttf" || ext_lower == "otf" || ext_lower == "ttc" {
                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                        let stem_clean: String = stem
                            .to_lowercase()
                            .chars()
                            .filter(|c| c.is_alphanumeric())
                            .collect();
                        for cand in candidates {
                            let cand_clean: String =
                                cand.chars().filter(|c| c.is_alphanumeric()).collect();
                            if stem_clean == cand_clean {
                                return Some(p);
                            }
                        }
                    }
                }
            }
        } else if p.is_dir() {
            subdirs.push(p);
        }
    }

    for subdir in subdirs {
        if let Some(found) = scan_dir_for_font(&subdir, candidates, depth + 1) {
            return Some(found);
        }
    }
    None
}

fn scan_dir_for_font_names(dir: &Path, depth: usize, fonts: &mut HashSet<String>) {
    if depth > 4 || !dir.is_dir() {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ext_lower == "ttf" || ext_lower == "otf" {
                        if let Some(name) = read_ttf_family_name(&p) {
                            if !name.trim().is_empty() {
                                fonts.insert(name);
                            }
                        }
                    }
                }
            } else if p.is_dir() {
                scan_dir_for_font_names(&p, depth + 1, fonts);
            }
        }
    }
}

pub fn get_available_fonts() -> Vec<String> {
    let mut lock = AVAILABLE_FONTS.lock().unwrap();
    if let Some(fonts) = lock.as_ref() {
        return fonts.clone();
    }

    let mut fonts = HashSet::new();
    let default_fonts = [
        "Arial",
        "Impact",
        "Roboto",
        "Montserrat",
        "Helvetica",
        "Bebas Neue",
        "Open Sans",
        "Futura",
        "Trebuchet MS",
        "Verdana",
        "Georgia",
        "Times New Roman",
        "Courier New",
        "Comic Sans MS",
    ];
    for f in default_fonts {
        fonts.insert(f.to_string());
    }

    let dirs = get_system_font_dirs();
    for dir in dirs {
        scan_dir_for_font_names(&dir, 0, &mut fonts);
    }

    let mut font_list: Vec<String> = fonts.into_iter().collect();
    font_list.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    *lock = Some(font_list.clone());
    font_list
}

pub fn try_load_system_font(ctx: &egui::Context, name: &str, bold: bool) -> bool {
    let key = if bold {
        format!("{}-Bold", name)
    } else {
        name.to_string()
    };

    if is_font_registered(&key) {
        return true;
    }

    {
        let mut loaded = LOADED_FONTS.lock().unwrap();
        let set = loaded.get_or_insert_with(HashSet::new);
        if set.contains(&key) {
            return false;
        }
        set.insert(key.clone());
    }

    let candidates = get_font_candidates(name, bold);
    let dirs = get_system_font_dirs();

    for dir in dirs {
        if let Some(path) = scan_dir_for_font(&dir, &candidates, 0) {
            if let Ok(bytes) = std::fs::read(&path) {
                register_font(ctx, &key, bytes);
                return true;
            }
        }
    }

    false
}

pub fn get_egui_font_id(
    ctx: &egui::Context,
    family_name: &str,
    size: f32,
    bold: bool,
) -> (egui::FontId, bool) {
    if family_name.is_empty() {
        return (egui::FontId::proportional(size), bold);
    }

    if bold {
        let bold_key = format!("{}-Bold", family_name);
        if is_font_registered(&bold_key) || try_load_system_font(ctx, family_name, true) {
            let fam = egui::FontFamily::Name(bold_key.into());
            if is_font_live(ctx, &fam) {
                return (egui::FontId::new(size, fam), false);
            }
            ctx.request_repaint();
        }
    }

    if is_font_registered(family_name) || try_load_system_font(ctx, family_name, false) {
        let fam = egui::FontFamily::Name(family_name.to_string().into());
        if is_font_live(ctx, &fam) {
            return (egui::FontId::new(size, fam), bold);
        }
        ctx.request_repaint();
    }

    if family_name.eq_ignore_ascii_case("monospace")
        || family_name.eq_ignore_ascii_case("courier")
        || family_name.eq_ignore_ascii_case("courier new")
    {
        (egui::FontId::monospace(size), bold)
    } else {
        (egui::FontId::proportional(size), bold)
    }
}

pub fn draw(
    ctx: &Context,
    canvas_painter: &egui::Painter,
    preview_rect: Rect,
    scale_factor: f32,
    vm: &mut EditorViewModel,
) {
    if let Some(sub) = vm.active_subtitle() {
        let max_steps = if sub.motion_blur > 0.0 { 3 } else { 0 };

        for step in (0..=max_steps).rev() {
            let offset_t = vm.current_time() - (step as f64 * 0.02 * sub.motion_blur as f64);
            if offset_t < sub.timeline_start && step > 0 {
                continue;
            }

            let state = sub.get_interpolated_state(offset_t, &vm.project.subtitles, 0);

            let opacity_mult = if step > 0 { 0.25 } else { 1.0 };
            let final_opacity = state.opacity * opacity_mult;

            if final_opacity <= 0.01 {
                continue;
            }

            let clip_rects = get_mask_clip_rects(sub, &state, preview_rect, scale_factor);
            if clip_rects.is_empty() {
                continue;
            }

            let font_size = (sub.font_size * state.scale * scale_factor).max(0.1);

            let center_x = preview_rect.center().x + state.x * scale_factor;
            let center_y = preview_rect.center().y + state.y * scale_factor;

            let rot_rad = state.rotation * std::f32::consts::PI / 180.0;

            let text_color = {
                let c = sub.color;
                egui::Color32::from_rgba_unmultiplied(
                    (c[0] * 255.0) as u8,
                    (c[1] * 255.0) as u8,
                    (c[2] * 255.0) as u8,
                    (c[3] * final_opacity * 255.0) as u8,
                )
            };

            let has_words = !sub.words.is_empty()
                && (sub.word_animation != WordAnimation::None
                    || sub.words.iter().any(|w| w.custom_color.is_some()));

            let current_t = vm.current_time();

            let apply_transform = |dx: f32, dy: f32| -> Pos2 {
                let rx = dx * rot_rad.cos() - dy * rot_rad.sin();
                let ry = dx * rot_rad.sin() + dy * rot_rad.cos();
                Pos2::new(center_x + rx, center_y + ry)
            };

            let mut final_text_size = Vec2::ZERO;

            for clip_rect in &clip_rects {
                let mut active_painter = canvas_painter.clone();
                active_painter.set_clip_rect(clip_rect.intersect(canvas_painter.clip_rect()));

                let create_job = |override_color: Option<egui::Color32>| -> (
                    std::sync::Arc<egui::Galley>,
                    bool,
                ) {
                    build_layout_job(
                        ctx,
                        sub,
                        has_words,
                        font_size,
                        text_color,
                        final_opacity,
                        override_color,
                        current_t,
                    )
                };

                let (galley, faux_bold) = create_job(None);
                let text_size = galley.size();
                final_text_size = text_size;

                let draw_projected_text =
                    |painter: &egui::Painter, g: std::sync::Arc<egui::Galley>, offset: Vec2| {
                        // Offset by the actual galley center to properly align the text inside its box
                        let dx = offset.x - g.rect.center().x;
                        let dy = offset.y - g.rect.center().y;
                        let pos = apply_transform(dx, dy);
                        let shape = egui::epaint::TextShape {
                            pos,
                            galley: g.clone(),
                            underline: Stroke::NONE,
                            override_text_color: None,
                            angle: rot_rad,
                            fallback_color: egui::Color32::WHITE,
                            opacity_factor: 1.0,
                        };
                        painter.add(shape);

                        if faux_bold {
                            let bold_off = Vec2::new(0.7 * scale_factor, 0.0);
                            let pos_b = apply_transform(dx + bold_off.x, dy + bold_off.y);
                            let shape_b = egui::epaint::TextShape {
                                pos: pos_b,
                                galley: g,
                                underline: Stroke::NONE,
                                override_text_color: None,
                                angle: rot_rad,
                                fallback_color: egui::Color32::WHITE,
                                opacity_factor: 1.0,
                            };
                            painter.add(shape_b);
                        }
                    };

                if sub.bg_box_enabled {
                    let pad = sub.bg_box_padding * scale_factor;
                    let w = text_size.x + pad * 2.0;
                    let h = text_size.y + pad * 2.0;
                    let r = sub.bg_box_radius;
                    let mut raw_pts = Vec::new();
                    let segments = 6;
                    let r_tl = (r[0] * scale_factor).min(w / 2.0).min(h / 2.0);
                    if r_tl > 0.0 {
                        for i in 0..=segments {
                            let a = std::f32::consts::PI
                                + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
                            raw_pts.push(Vec2::new(
                                -w / 2.0 + r_tl + r_tl * a.cos(),
                                -h / 2.0 + r_tl + r_tl * a.sin(),
                            ));
                        }
                    } else {
                        raw_pts.push(Vec2::new(-w / 2.0, -h / 2.0));
                    }
                    let r_tr = (r[1] * scale_factor).min(w / 2.0).min(h / 2.0);
                    if r_tr > 0.0 {
                        for i in 0..=segments {
                            let a = -std::f32::consts::FRAC_PI_2
                                + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
                            raw_pts.push(Vec2::new(
                                w / 2.0 - r_tr + r_tr * a.cos(),
                                -h / 2.0 + r_tr + r_tr * a.sin(),
                            ));
                        }
                    } else {
                        raw_pts.push(Vec2::new(w / 2.0, -h / 2.0));
                    }
                    let r_br = (r[2] * scale_factor).min(w / 2.0).min(h / 2.0);
                    if r_br > 0.0 {
                        for i in 0..=segments {
                            let a =
                                0.0 + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
                            raw_pts.push(Vec2::new(
                                w / 2.0 - r_br + r_br * a.cos(),
                                h / 2.0 - r_br + r_br * a.sin(),
                            ));
                        }
                    } else {
                        raw_pts.push(Vec2::new(w / 2.0, h / 2.0));
                    }
                    let r_bl = (r[3] * scale_factor).min(w / 2.0).min(h / 2.0);
                    if r_bl > 0.0 {
                        for i in 0..=segments {
                            let a = std::f32::consts::FRAC_PI_2
                                + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
                            raw_pts.push(Vec2::new(
                                -w / 2.0 + r_bl + r_bl * a.cos(),
                                h / 2.0 - r_bl + r_bl * a.sin(),
                            ));
                        }
                    } else {
                        raw_pts.push(Vec2::new(-w / 2.0, h / 2.0));
                    }
                    let pts = raw_pts
                        .iter()
                        .map(|v| apply_transform(v.x, v.y))
                        .collect::<Vec<_>>();
                    let bc = sub.bg_box_color;
                    let box_color = egui::Color32::from_rgba_unmultiplied(
                        (bc[0] * 255.0) as u8,
                        (bc[1] * 255.0) as u8,
                        (bc[2] * 255.0) as u8,
                        (bc[3] * final_opacity * 255.0) as u8,
                    );
                    active_painter.add(egui::Shape::convex_polygon(
                        pts,
                        box_color,
                        Stroke::new(1.0_f32, box_color),
                    ));
                }

                if sub.bloom.enabled {
                    let b_col = sub.bloom.color;
                    let max_r = sub.bloom.radius * scale_factor;
                    if max_r > 0.0 {
                        let passes = (max_r / 4.0).clamp(3.0, 10.0) as usize;
                        let blur_r = max_r / passes as f32;
                        for b_step in 1..=passes {
                            let r = b_step as f32 * blur_r;
                            let alpha_frac = (passes + 1 - b_step) as f32 / (passes + 1) as f32;
                            let step_col = egui::Color32::from_rgba_unmultiplied(
                                (b_col[0] * 255.) as u8,
                                (b_col[1] * 255.) as u8,
                                (b_col[2] * 255.) as u8,
                                (b_col[3]
                                    * 255.
                                    * final_opacity
                                    * sub.bloom.intensity
                                    * alpha_frac
                                    * (2.0 / passes as f32))
                                    .clamp(0.0, 255.0) as u8,
                            );
                            let samples = (r * 2.0).clamp(6.0, 20.0) as usize;
                            for i in 0..samples {
                                let angle = (i as f32 / samples as f32) * std::f32::consts::TAU;
                                let dx = angle.cos() * r;
                                let dy = angle.sin() * r;
                                let (g, _) = create_job(Some(step_col));
                                draw_projected_text(&active_painter, g, Vec2::new(dx, dy));
                            }
                        }
                    }
                }

                if sub.additional_strokes_enabled {
                    let mut sorted_strokes = sub.additional_strokes.clone();
                    sorted_strokes.sort_by(|a, b| b.width.partial_cmp(&a.width).unwrap());
                    for stroke in &sorted_strokes {
                        if stroke.enabled && stroke.width > 0.0 {
                            let sw = stroke.width * scale_factor;
                            let stc = stroke.color;
                            let stroke_color = egui::Color32::from_rgba_unmultiplied(
                                (stc[0] * 255.0) as u8,
                                (stc[1] * 255.0) as u8,
                                (stc[2] * 255.0) as u8,
                                (stc[3] * final_opacity * 255.0) as u8,
                            );
                            let samples = (sw * 2.5).clamp(8.0, 32.0) as usize;
                            for i in 0..samples {
                                let angle = (i as f32 / samples as f32) * std::f32::consts::TAU;
                                let dx = angle.cos() * sw;
                                let dy = angle.sin() * sw;
                                let (g, _) = create_job(Some(stroke_color));
                                draw_projected_text(&active_painter, g, Vec2::new(dx, dy));
                            }
                        }
                    }
                }

                if sub.shadow_enabled {
                    let sc = sub.shadow_color;
                    let s_col = egui::Color32::from_rgba_unmultiplied(
                        (sc[0] * 255.) as u8,
                        (sc[1] * 255.) as u8,
                        (sc[2] * 255.) as u8,
                        (sc[3] * final_opacity * 255.) as u8,
                    );
                    let raw_off = Vec2::new(
                        sub.shadow_offset[0] * scale_factor,
                        sub.shadow_offset[1] * scale_factor,
                    );
                    let roff_x = raw_off.x * rot_rad.cos() - raw_off.y * rot_rad.sin();
                    let roff_y = raw_off.x * rot_rad.sin() + raw_off.y * rot_rad.cos();
                    let offset = Vec2::new(roff_x, roff_y);

                    if sub.shadow_blur > 0.0 {
                        let blur_steps =
                            (sub.shadow_blur * scale_factor / 2.0).clamp(2.0, 10.0) as usize;
                        let blur_r = sub.shadow_blur * scale_factor / blur_steps as f32;
                        for b_step in 1..=blur_steps {
                            let r = b_step as f32 * blur_r;
                            let alpha_frac =
                                (blur_steps + 1 - b_step) as f32 / (blur_steps + 1) as f32;
                            let step_col = egui::Color32::from_rgba_unmultiplied(
                                s_col.r(),
                                s_col.g(),
                                s_col.b(),
                                (s_col.a() as f32 * alpha_frac * (2.0 / blur_steps as f32))
                                    .clamp(0.0, 255.0) as u8,
                            );
                            let samples = (r * 2.0).clamp(6.0, 20.0) as usize;
                            for i in 0..samples {
                                let angle = (i as f32 / samples as f32) * std::f32::consts::TAU;
                                let dx = angle.cos() * r;
                                let dy = angle.sin() * r;
                                let (g, _) = create_job(Some(step_col));
                                draw_projected_text(&active_painter, g, offset + Vec2::new(dx, dy));
                            }
                        }
                    } else {
                        let (g, _) = create_job(Some(s_col));
                        draw_projected_text(&active_painter, g, offset);
                    }
                }

                if sub.stroke_enabled && sub.stroke_width > 0.0 {
                    let sw = sub.stroke_width * scale_factor;
                    let stc = sub.stroke_color;
                    let stroke_color = egui::Color32::from_rgba_unmultiplied(
                        (stc[0] * 255.) as u8,
                        (stc[1] * 255.) as u8,
                        (stc[2] * 255.) as u8,
                        (stc[3] * final_opacity * 255.) as u8,
                    );
                    let samples = (sw * 2.5).clamp(8.0, 32.0) as usize;
                    for i in 0..samples {
                        let angle = (i as f32 / samples as f32) * std::f32::consts::TAU;
                        let dx = angle.cos() * sw;
                        let dy = angle.sin() * sw;
                        let (g, _) = create_job(Some(stroke_color));
                        draw_projected_text(&active_painter, g, Vec2::new(dx, dy));
                    }
                }

                if sub.gradient_enabled {
                    let gc = sub.gradient_color;
                    let top_color = egui::Color32::from_rgba_unmultiplied(
                        (gc[0] * 255.) as u8,
                        (gc[1] * 255.) as u8,
                        (gc[2] * 255.) as u8,
                        (gc[3] * final_opacity * 255.) as u8,
                    );
                    let (g, _) = create_job(Some(top_color));
                    draw_projected_text(&active_painter, g, Vec2::ZERO);
                } else {
                    let (galley2, _) = create_job(None);
                    draw_projected_text(&active_painter, galley2, Vec2::ZERO);
                }
            }

            if step == 0 && vm.selected_id.as_deref() == Some(&sub.id) {
                let text_size = final_text_size;
                let pad = if sub.bg_box_enabled {
                    sub.bg_box_padding * scale_factor
                } else {
                    4.0
                };
                let w = text_size.x + pad * 2.0;
                let h = text_size.y + pad * 2.0;
                let pts = [
                    Vec2::new(-w / 2.0, -h / 2.0),
                    Vec2::new(w / 2.0, -h / 2.0),
                    Vec2::new(w / 2.0, h / 2.0),
                    Vec2::new(-w / 2.0, h / 2.0),
                ]
                .iter()
                .map(|v| apply_transform(v.x, v.y))
                .collect::<Vec<_>>();
                let shape = egui::Shape::convex_polygon(
                    pts,
                    egui::Color32::TRANSPARENT,
                    Stroke::new(1.5_f32, ACCENT_CYAN),
                );
                canvas_painter.add(shape);
            }
        }
    }
}

fn get_mask_clip_rects(
    sub: &crate::models::types::subtitle::Subtitle,
    state: &crate::models::types::animation::InterpolatedState,
    preview_rect: Rect,
    scale_factor: f32,
) -> Vec<Rect> {
    if sub.mask_type == MaskType::None {
        return vec![preview_rect];
    }

    let mc_x = preview_rect.center().x + state.mask_center[0] * scale_factor;
    let mc_y = preview_rect.center().y + state.mask_center[1] * scale_factor;
    let rot = state.mask_rotation.to_radians();

    let rot_pt = |dx: f32, dy: f32| -> Pos2 {
        let rx = dx * rot.cos() - dy * rot.sin();
        let ry = dx * rot.sin() + dy * rot.cos();
        Pos2::new(mc_x + rx, mc_y + ry)
    };

    match sub.mask_type {
        MaskType::Rectangle | MaskType::Circle => {
            let hw = if sub.mask_type == MaskType::Circle {
                state.mask_size[0] * scale_factor
            } else {
                state.mask_size[0] * 0.5 * scale_factor
            };
            let hh = if sub.mask_type == MaskType::Circle {
                state.mask_size[0] * scale_factor
            } else {
                state.mask_size[1] * 0.5 * scale_factor
            };

            let corners = [
                rot_pt(-hw, -hh),
                rot_pt(hw, -hh),
                rot_pt(hw, hh),
                rot_pt(-hw, hh),
            ];
            let min_x = corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
            let max_x = corners
                .iter()
                .map(|p| p.x)
                .fold(f32::NEG_INFINITY, f32::max);
            let min_y = corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
            let max_y = corners
                .iter()
                .map(|p| p.y)
                .fold(f32::NEG_INFINITY, f32::max);

            let inner = Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));

            if sub.mask_invert {
                let mut rects = Vec::new();
                if inner.min.y > preview_rect.min.y {
                    let r = Rect::from_min_max(
                        Pos2::new(preview_rect.min.x, preview_rect.min.y),
                        Pos2::new(preview_rect.max.x, inner.min.y.min(preview_rect.max.y)),
                    );
                    if r.is_positive() {
                        rects.push(r);
                    }
                }
                if inner.max.y < preview_rect.max.y {
                    let r = Rect::from_min_max(
                        Pos2::new(preview_rect.min.x, inner.max.y.max(preview_rect.min.y)),
                        Pos2::new(preview_rect.max.x, preview_rect.max.y),
                    );
                    if r.is_positive() {
                        rects.push(r);
                    }
                }
                if inner.min.x > preview_rect.min.x {
                    let r = Rect::from_min_max(
                        Pos2::new(preview_rect.min.x, inner.min.y.max(preview_rect.min.y)),
                        Pos2::new(
                            inner.min.x.min(preview_rect.max.x),
                            inner.max.y.min(preview_rect.max.y),
                        ),
                    );
                    if r.is_positive() {
                        rects.push(r);
                    }
                }
                if inner.max.x < preview_rect.max.x {
                    let r = Rect::from_min_max(
                        Pos2::new(
                            inner.max.x.max(preview_rect.min.x),
                            inner.min.y.max(preview_rect.min.y),
                        ),
                        Pos2::new(preview_rect.max.x, inner.max.y.min(preview_rect.max.y)),
                    );
                    if r.is_positive() {
                        rects.push(r);
                    }
                }
                rects
            } else {
                let r = inner.intersect(preview_rect);
                if r.is_positive() {
                    vec![r]
                } else {
                    vec![]
                }
            }
        }
        MaskType::Straight => {
            let mut valid_pts = Vec::new();
            let sin_r = rot.sin();
            let cos_r = rot.cos();

            let keep = |x: f32, y: f32| -> bool {
                let dx = x - mc_x;
                let dy = y - mc_y;
                let local_y = -dx * sin_r + dy * cos_r;
                if sub.mask_invert {
                    local_y <= 0.0
                } else {
                    local_y >= 0.0
                }
            };

            let corners = [
                preview_rect.left_top(),
                preview_rect.right_top(),
                preview_rect.right_bottom(),
                preview_rect.left_bottom(),
            ];
            for p in &corners {
                if keep(p.x, p.y) {
                    valid_pts.push(*p);
                }
            }

            if cos_r.abs() > 1e-5 {
                let tan_r = sin_r / cos_r;
                for &x in &[preview_rect.min.x, preview_rect.max.x] {
                    let dy = (x - mc_x) * tan_r;
                    let y = mc_y + dy;
                    if y >= preview_rect.min.y && y <= preview_rect.max.y {
                        valid_pts.push(Pos2::new(x, y));
                    }
                }
            }

            if sin_r.abs() > 1e-5 {
                let cot_r = cos_r / sin_r;
                for &y in &[preview_rect.min.y, preview_rect.max.y] {
                    let dx = (y - mc_y) * cot_r;
                    let x = mc_x + dx;
                    if x >= preview_rect.min.x && x <= preview_rect.max.x {
                        valid_pts.push(Pos2::new(x, y));
                    }
                }
            }

            if valid_pts.is_empty() {
                return vec![];
            }

            let min_x = valid_pts.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
            let max_x = valid_pts
                .iter()
                .map(|p| p.x)
                .fold(f32::NEG_INFINITY, f32::max);
            let min_y = valid_pts.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
            let max_y = valid_pts
                .iter()
                .map(|p| p.y)
                .fold(f32::NEG_INFINITY, f32::max);

            let r = Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));
            if r.is_positive() {
                vec![r.intersect(preview_rect)]
            } else {
                vec![]
            }
        }
        MaskType::None => vec![preview_rect],
    }
}

fn build_layout_job(
    ctx: &Context,
    sub: &crate::models::types::subtitle::Subtitle,
    has_words: bool,
    font_size: f32,
    text_color: egui::Color32,
    final_opacity: f32,
    override_color: Option<egui::Color32>,
    current_time: f64,
) -> (std::sync::Arc<egui::Galley>, bool) {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    job.justify = false;

    job.halign = match sub.text_align {
        crate::models::types::subtitle::TextAlign::Left => egui::Align::LEFT,
        crate::models::types::subtitle::TextAlign::Center => egui::Align::Center,
        crate::models::types::subtitle::TextAlign::Right => egui::Align::RIGHT,
    };

    let mut faux_bold = false;

    if has_words {
        for (i, word) in sub.words.iter().enumerate() {
            let is_active = current_time >= word.start && current_time <= word.end;
            let mut word_color = text_color;
            let mut word_scale = 1.0;
            let mut word_opacity = 1.0;

            match &sub.word_animation {
                WordAnimation::KaraokeHighlight { color } => {
                    if is_active {
                        word_color = egui::Color32::from_rgba_unmultiplied(
                            (color[0] * 255.) as u8,
                            (color[1] * 255.) as u8,
                            (color[2] * 255.) as u8,
                            (color[3] * final_opacity * 255.) as u8,
                        );
                    }
                }
                WordAnimation::KaraokePop { scale } => {
                    if is_active {
                        word_scale = *scale;
                        word_color = egui::Color32::from_rgba_unmultiplied(
                            255,
                            220,
                            0,
                            (final_opacity * 255.) as u8,
                        );
                    }
                }
                WordAnimation::CascadeFade => {
                    if current_time < word.start {
                        word_opacity = 0.0;
                    } else if is_active {
                        word_opacity = ((current_time - word.start)
                            / (word.end - word.start).max(0.01))
                            as f32;
                    }
                    word_color = egui::Color32::from_rgba_unmultiplied(
                        word_color.r(),
                        word_color.g(),
                        word_color.b(),
                        (word_color.a() as f32 * word_opacity) as u8,
                    );
                }
                WordAnimation::None => {}
            }

            if let Some(c) = word.custom_color {
                word_color = egui::Color32::from_rgba_unmultiplied(
                    (c[0] * 255.) as u8,
                    (c[1] * 255.) as u8,
                    (c[2] * 255.) as u8,
                    (c[3] * final_opacity * 255.) as u8,
                );
            }

            let final_col = if let Some(oc) = override_color {
                egui::Color32::from_rgba_unmultiplied(
                    oc.r(),
                    oc.g(),
                    oc.b(),
                    (oc.a() as f32 * word_opacity) as u8,
                )
            } else {
                word_color
            };

            let (font_id, fb) =
                get_egui_font_id(ctx, &sub.font_family, font_size * word_scale, sub.bold);
            faux_bold = fb;

            job.append(
                &word.text,
                0.0,
                egui::text::TextFormat {
                    font_id,
                    color: final_col,
                    italics: sub.italic,
                    extra_letter_spacing: sub.letter_spacing,
                    ..Default::default()
                },
            );

            if i < sub.words.len() - 1 {
                let spc_col = override_color.unwrap_or(text_color);
                let (spc_font_id, _) = get_egui_font_id(ctx, &sub.font_family, font_size, sub.bold);
                job.append(
                    " ",
                    0.0,
                    egui::text::TextFormat {
                        font_id: spc_font_id,
                        color: spc_col,
                        italics: sub.italic,
                        extra_letter_spacing: sub.letter_spacing,
                        ..Default::default()
                    },
                );
            }
        }
    } else {
        let (font_id, fb) = get_egui_font_id(ctx, &sub.font_family, font_size, sub.bold);
        faux_bold = fb;
        job.append(
            &sub.text,
            0.0,
            egui::text::TextFormat {
                font_id,
                color: override_color.unwrap_or(text_color),
                italics: sub.italic,
                extra_letter_spacing: sub.letter_spacing,
                ..Default::default()
            },
        );
    }

    (ctx.fonts(|f| f.layout_job(job)), faux_bold)
}
