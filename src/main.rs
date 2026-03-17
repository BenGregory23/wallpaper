use gpui::*;

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

struct Wallpaper {
    source: ImageSource,
    selected: &mut bool,
}

impl Render for Wallpaper {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .w_96()
            .justify_center()
            .child(self.render_wallpaper_item())
            .on_mouse_down(
                MouseButton::Left,
                _cx.listener(|view, _event, cx| {
                    view.selected = !view.selected;

                    // This is the magic line that makes the width change happen!
                    cx.notify();

                    println!("Clicked! Selected is now {}", view.selected);
                }),
            )
    }
}

impl Wallpaper {
    fn render_wallpaper_item(&self) -> impl IntoElement {
        div()
            .h_full()
            .w(px(100.0))
            .bg(rgb(0x1e1e1e))
            .overflow_hidden()
            .rounded_lg()
            .child(
                img(self.source.clone())
                    .size_full()
                    // CRITICAL: This crops the image instead of stretching it
                    .object_fit(ObjectFit::Cover),
            )
    }
}

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                traffic_light_position: None,
                title: None,
                appears_transparent: true,
            }),
            window_background: WindowBackgroundAppearance::Transparent,
            ..Default::default()
        };

        cx.open_window(options, |_, cx| {
            cx.new(|_cx| Wallpaper {
                source: "wallpapers/test.jpg".into(),
                selected: false,
            })
        })
        .unwrap();
    });
}
