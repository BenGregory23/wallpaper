use gpui::*;
use std::{path, process::Command};

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        // This is a simple implementation that reads from the local 'assets' folder
        let full_path = std::path::Path::new("assets").join(path);
        if full_path.exists() {
            let bytes = std::fs::read(full_path)?;
            Ok(Some(std::borrow::Cow::Owned(bytes)))
        } else {
            Ok(None)
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = Vec::new();
        let dir_path = std::path::Path::new("assets").join(path);

        for entry in std::fs::read_dir(dir_path)? {
            let entry = entry?;
            // 1. Get the OsString
            let file_name = entry.file_name();

            // 2. Convert it to a String (owned), then to SharedString
            if let Some(name_str) = file_name.to_str() {
                paths.push(SharedString::from(name_str.to_string()));
            }
        }

        Ok(paths)
    }
}

struct WallpaperGallery {
    base_url: SharedString,
    sources: Vec<ImageSource>,
}

impl WallpaperGallery {
    fn render_wallpaper_item(
        &self,
        source: ImageSource,
        base_url: SharedString,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let my_source = source.clone();

        div()
            .h_1_2()
            .w(px(100.0))
            .bg(rgb(0x1e1e1e))
            .overflow_hidden()
            .rounded_lg()
            .hover(|this| this.cursor_pointer().border_4().border_color(rgb(0xe0e0e0)))
            .on_mouse_down(MouseButton::Left, move |_event, _window, _cx| {
                set_wallpaper(my_source.clone(), base_url.clone());
            })
            .child(
                img(source.clone())
                    .size_full()
                    .object_fit(ObjectFit::Cover)
                    .rounded_lg()
                    .overflow_hidden(),
            )
    }
}

impl Render for WallpaperGallery {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .gap_4()
            .p_4()
            .justify_center()
            .items_center()
            // Map every source in our list to a UI element
            .children(self.sources.iter().map(|source| {
                self.render_wallpaper_item(source.clone(), self.base_url.clone(), _cx)
            }))
    }
}

fn set_wallpaper(source: ImageSource, base_url: SharedString) {
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
        let complete_path = format!("{}{}", base_url, path);
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!(" plasma-apply-wallpaperimage {}", complete_path))
            .spawn();
    } else {
        eprintln!("Source is a memory buffer or closure; no path available.");
    }
}

actions!(window, [Quit]);

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
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

        let asset_paths = cx.asset_source().list("wallpapers").unwrap_or_default();
        let sources: Vec<ImageSource> = asset_paths
            .into_iter()
            .map(|path| format!("wallpapers/{}", path).into())
            .collect();

        cx.open_window(options, |_, cx| {
            cx.new(|_cx| WallpaperGallery {
                base_url: "/home/bengregory/Documents/programming/wallpaper/assets/".into(),
                sources: sources,
            })
        })
        .unwrap();

        cx.activate(true);
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
    });
}
