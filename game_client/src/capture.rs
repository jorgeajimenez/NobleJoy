use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::PrimaryWindow;

pub const VIEWPORT_CAPTURE_PATH: &str = "viewport_capture.png";

pub fn capture_viewport_to_disk(
    main_window: &Query<Entity, With<PrimaryWindow>>,
    screenshot_manager: &mut ResMut<ScreenshotManager>,
) -> Option<String> {
    if let Ok(window_entity) = main_window.get_single() {
        let path = VIEWPORT_CAPTURE_PATH.to_string();
        let _ = screenshot_manager.save_screenshot_to_disk(window_entity, &path);
        return Some(path);
    }
    None
}
