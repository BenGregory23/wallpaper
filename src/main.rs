use gpui::*;
mod image_compressor;

use std::path::PathBuf;
use std::{process::Command, time::Duration, time::Instant};

use crate::image_compressor::compress_directory;

struct Wallpaper {
    id: i32,
    default: ImageSource,
    compressed: ImageSource,
}

pub struct WallpaperModel {
    sources: Vec<Wallpaper>,
    selected: Option<i32>,
}

struct WallpaperGallery {
    model: Entity<WallpaperModel>,
}

impl Render for WallpaperGallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.clone();
        let model_data = self.model.read(cx);
        let current_selected = model_data.selected.clone();

        div()
            .id("wallpaper-scroll-container")
            .flex()
            .bg(hsla(175.0, 0.0, 0.0, 0.45))
            .overflow_x_scroll()
            .size_full()
            .gap_4()
            .p_4()
            .items_center()
            .children(model_data.sources.iter().map(|s| {
                let is_active = current_selected.is_some() && current_selected.unwrap().eq(&s.id);
                wallpaper_item(s, is_active, model.clone())
            }))
    }
}

fn wallpaper_item(
    wallpaper: &Wallpaper,
    is_active: bool,
    model: Entity<WallpaperModel>,
) -> impl IntoElement {
    let default = wallpaper.default.clone();
    let compressed = wallpaper.compressed.clone();
    let id = wallpaper.id.clone();

    // Widths
    let collapsed_width = 100.0;
    let expanded_width = 400.0;

    let target_width = if is_active {
        expanded_width
    } else {
        collapsed_width
    };

    let container = div()
        .h(px(300.0))
        .w(px(target_width))
        .flex_none()
        .bg(rgb(0x1e1e1e))
        .border_2()
        .border_color(hsla(0.0, 0.0, 0.0, 0.0))
        .hover(|style| {
            style
                .cursor_pointer()
                .size_full()
                .border_4()
                .border_color(rgb(0xe0e0e0))
        })
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model.update(cx, |state, _cx| {
                state.selected = Some(id);
            });
            set_wallpaper(default.clone());
        })
        .child(img(compressed).size_full().object_fit(ObjectFit::Cover))
        .overflow_hidden();

    if is_active {
        let anim_id: SharedString = format!("grow_{}", wallpaper.id).into();
        container
            .with_animation(
                anim_id,
                Animation::new(Duration::from_millis(200)).with_easing(ease_in_out),
                move |this, delta| {
                    let current_width =
                        collapsed_width + (expanded_width - collapsed_width) * delta;
                    this.w(px(current_width))
                },
            )
            .into_any_element()
    } else {
        container.w(px(collapsed_width)).into_any_element()
    }
}

fn set_wallpaper(source: ImageSource) {
    let path_str: Option<SharedString> = match source {
        ImageSource::Resource(res) => match res {
            Resource::Embedded(s) => Some(s),
            // If Resource has a Path/File variant, handle it here:
            Resource::Path(p) => Some(p.to_string_lossy().to_string().into()),
            _ => None,
        },
        _ => None,
    };

    if let Some(path) = path_str {
        let complete_path = format!("{}", path);
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!(" plasma-apply-wallpaperimage {}", complete_path))
            .spawn();
    } else {
        eprintln!("Source is a memory buffer or closure; no path available.");
    }
}

fn load_wallpaper_paths(base_url: &std::path::Path) -> Result<Vec<SharedString>> {
    let mut paths = Vec::new();
    let dir_path = std::path::Path::new(&base_url);

    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let file_name = entry.file_name();

        if let Some(name_str) = file_name.to_str() {
            paths.push(SharedString::from(name_str.to_string()));
        }
    }

    Ok(paths)
}

actions!(window, [Quit]);

fn main() {
    let base_directory = "/home/bengregory/Documents/programming/wallpaper/assets/wallpapers/";

    // compress images if no images compressed
    let compressed_dir = std::path::Path::new(&base_directory).join("compressed");
    if !compressed_dir.exists() {
        let before = Instant::now();
        let _ = compress_directory(&base_directory);
        let after = Instant::now();
        println!("{:?}", after.duration_since(before));
    }

    Application::new().run(move |cx: &mut App| {
        gpui_component::init(cx);
        let bounds = Bounds::centered(None, size(px(1920.0), px(1080.0)), cx);
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                traffic_light_position: None,
                title: None,
                appears_transparent: true,
            }),
            window_bounds: Some(WindowBounds::Fullscreen(bounds)),
            kind: WindowKind::PopUp,
            window_background: WindowBackgroundAppearance::Transparent,
            ..Default::default()
        };

        let asset_paths = load_wallpaper_paths(&compressed_dir).unwrap();
        let mut index = 0;
        let wallpapers: Vec<Wallpaper> = asset_paths
            .into_iter()
            .map(|a| {
                index = index + 1;
                Wallpaper {
                    id: index.clone(),
                    default: PathBuf::from(base_directory).join(&a.as_str()).into(),
                    compressed: PathBuf::from(&compressed_dir).join(&a.as_str()).into(),
                }
            })
            .collect();

        let wallpaper_model = cx.new(|_cx| WallpaperModel {
            sources: wallpapers,
            selected: Option::None,
        });

        let _ = cx
            .open_window(options, |_, cx| {
                cx.new(|_cx| WallpaperGallery {
                    model: wallpaper_model.clone(),
                })
            })
            .expect("Failed to open window");

        cx.activate(true);
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([KeyBinding::new("ctrl-q", Quit, None)]);
    });
}
