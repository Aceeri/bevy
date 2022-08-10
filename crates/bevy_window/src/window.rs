use bevy_ecs::system::Resource;
use std::borrow::Cow;

use bevy_ecs::{
    entity::Entity,
    prelude::{Bundle, Component, ReflectComponent},
    query::WorldQuery,
};
use bevy_math::{DVec2, IVec2, UVec2, Vec2};
use bevy_reflect::{FromReflect, Reflect};
use bevy_utils::{tracing::warn, Uuid};
use raw_window_handle::RawWindowHandle;

use crate::raw_window_handle::RawWindowHandleWrapper;
use crate::CursorIcon;

/// Presentation mode for a window.
///
/// The presentation mode specifies when a frame is presented to the window. The `Fifo`
/// option corresponds to a traditional `VSync`, where the framerate is capped by the
/// display refresh rate. Both `Immediate` and `Mailbox` are low-latency and are not
/// capped by the refresh rate, but may not be available on all platforms. Tearing
/// may be observed with `Immediate` mode, but will not be observed with `Mailbox` or
/// `Fifo`.
///
/// `Immediate` or `Mailbox` will gracefully fallback to `Fifo` when unavailable.
///
/// The presentation mode may be declared in the [`WindowDescriptor`](WindowDescriptor::present_mode)
/// or updated on a [`Window`](Window::set_present_mode).
#[repr(C)]
#[derive(Default, Copy, Clone, Component, Debug, PartialEq, Eq, Hash, Reflect)]
#[doc(alias = "vsync")]
#[reflect(Component)]
pub enum PresentMode {
    /// Chooses FifoRelaxed -> Fifo based on availability.
    ///
    /// Because of the fallback behavior, it is supported everywhere.
    AutoVsync = 0,
    /// Chooses Immediate -> Mailbox -> Fifo (on web) based on availability.
    ///
    /// Because of the fallback behavior, it is supported everywhere.
    AutoNoVsync = 1,
    /// The presentation engine does **not** wait for a vertical blanking period and
    /// the request is presented immediately. This is a low-latency presentation mode,
    /// but visible tearing may be observed. Will fallback to `Fifo` if unavailable on the
    /// selected platform and backend. Not optimal for mobile.
    ///
    /// Selecting this variant will panic if not supported, it is preferred to use
    /// [`PresentMode::AutoNoVsync`].
    Immediate = 2,
    /// The presentation engine waits for the next vertical blanking period to update
    /// the current image, but frames may be submitted without delay. This is a low-latency
    /// presentation mode and visible tearing will **not** be observed. Will fallback to `Fifo`
    /// if unavailable on the selected platform and backend. Not optimal for mobile.
    ///
    /// Selecting this variant will panic if not supported, it is preferred to use
    /// [`PresentMode::AutoNoVsync`].
    Mailbox = 3,
    /// The presentation engine waits for the next vertical blanking period to update
    /// the current image. The framerate will be capped at the display refresh rate,
    /// corresponding to the `VSync`. Tearing cannot be observed. Optimal for mobile.
    #[default]
    Fifo = 4, // NOTE: The explicit ordinal values mirror wgpu.
}

/// Defines the way a window is displayed
#[derive(Default, Debug, Component, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub enum WindowMode {
    /// Creates a window that uses the given size
    #[default]
    Windowed,
    /// Creates a borderless window that uses the full size of the screen
    BorderlessFullscreen,
    /// Creates a fullscreen window that will render at desktop resolution. The app will use the closest supported size
    /// from the given size and scale it to fit the screen.
    SizedFullscreen,
    /// Creates a fullscreen window that uses the maximum supported size
    Fullscreen,
}

/// Define how a window will be created and how it will behave.
#[derive(Default, Bundle, Debug, Clone)]
pub struct WindowBundle {
    pub window: Window,
    pub cursor: Cursor,
    pub cursor_position: CursorPosition,
    pub present_mode: PresentMode,
    pub mode: WindowMode,
    pub position: WindowPosition,
    pub resolution: WindowResolution,
    pub title: WindowTitle,
    // Maybe default this when using wasm?
    //pub canvas: WindowCanvas,
    pub resize_constraints: WindowResizeConstraints,
    pub resizable: WindowResizable,
    pub decorations: WindowDecorations,
    pub transparency: WindowTransparency,
    pub focus: WindowFocus,
}

#[derive(WorldQuery)]
pub struct WindowComponents<'a> {
    pub entity: Entity,
    pub window: &'a Window,
    pub cursor: &'a Cursor,
    pub cursor_position: &'a CursorPosition,
    pub present_mode: &'a PresentMode,
    pub window_mode: &'a WindowMode,
    pub position: &'a WindowPosition,
    pub resolution: &'a WindowResolution,
    pub title: &'a WindowTitle,
    pub resize_constraints: &'a WindowResizeConstraints,
    pub resizable: &'a WindowResizable,
    pub decorations: &'a WindowDecorations,
    pub transparency: &'a WindowTransparency,
    pub focus: &'a WindowFocus,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct WindowComponentsMut<'a> {
    pub entity: Entity,
    pub window: &'a mut Window,
    pub cursor: &'a mut Cursor,
    pub cursor_position: &'a mut CursorPosition,
    pub present_mode: &'a mut PresentMode,
    pub window_mode: &'a mut WindowMode,
    pub position: &'a mut WindowPosition,
    pub resolution: &'a mut WindowResolution,
    pub title: &'a mut WindowTitle,
    pub resize_constraints: &'a mut WindowResizeConstraints,
    pub resizable: &'a mut WindowResizable,
    pub decorations: &'a mut WindowDecorations,
    pub transparency: &'a mut WindowTransparency,
    pub focus: &'a mut WindowFocus,
}

/// The size limits on a window.
///
/// These values are measured in logical pixels, so the user's
/// scale factor does affect the size limits on the window.
/// Please note that if the window is resizable, then when the window is
/// maximized it may have a size outside of these limits. The functionality
/// required to disable maximizing is not yet exposed by winit.
#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct WindowResizeConstraints {
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
}

impl Default for WindowResizeConstraints {
    fn default() -> Self {
        Self {
            min_width: 180.,
            min_height: 120.,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        }
    }
}

impl WindowResizeConstraints {
    #[must_use]
    pub fn check_constraints(&self) -> Self {
        let WindowResizeConstraints {
            mut min_width,
            mut min_height,
            mut max_width,
            mut max_height,
        } = self;
        min_width = min_width.max(1.);
        min_height = min_height.max(1.);
        if max_width < min_width {
            warn!(
                "The given maximum width {} is smaller than the minimum width {}",
                max_width, min_width
            );
            max_width = min_width;
        }
        if max_height < min_height {
            warn!(
                "The given maximum height {} is smaller than the minimum height {}",
                max_height, min_height
            );
            max_height = min_height;
        }
        WindowResizeConstraints {
            min_width,
            min_height,
            max_width,
            max_height,
        }
    }
}

/// A marker component on an entity that is a window
#[derive(Default, Debug, Component, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct Window;

#[derive(Debug, Component, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct Cursor {
    icon: CursorIcon,
    visible: bool,
    locked: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            icon: CursorIcon::Default,
            visible: true,
            locked: false,
        }
    }
}

impl Cursor {
    pub fn new(icon: CursorIcon, visible: bool, locked: bool) -> Self {
        Self {
            icon,
            visible,
            locked,
        }
    }

    #[inline]
    pub fn icon(&self) -> CursorIcon {
        self.icon
    }

    #[inline]
    pub fn visible(&self) -> bool {
        self.visible
    }

    #[inline]
    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn set_icon(&mut self, icon: CursorIcon) {
        self.icon = icon;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }
}

#[derive(Default, Debug, Component, Clone, Reflect)]
#[reflect(Component)]
pub struct CursorPosition {
    /// Cursor position if it is inside of the window.
    physical_cursor_position: Option<DVec2>,
}

impl CursorPosition {
    pub fn new(physical_cursor_position: Option<DVec2>) -> Self {
        Self {
            physical_cursor_position,
        }
    }

    /// The current mouse position, in physical pixels.
    #[inline]
    pub fn physical_position(&self) -> Option<DVec2> {
        self.physical_cursor_position
    }

    pub fn set(&mut self, position: Option<DVec2>) {
        self.physical_cursor_position = position;
    }
}

#[derive(Component)]
pub struct WindowHandle {
    raw_window_handle: RawWindowHandleWrapper,
}

impl WindowHandle {
    pub fn new(raw_window_handle: RawWindowHandle) -> Self {
        Self {
            raw_window_handle: RawWindowHandleWrapper::new(raw_window_handle),
        }
    }

    pub fn raw_window_handle(&self) -> RawWindowHandleWrapper {
        self.raw_window_handle.clone()
    }
}

/// Defines where window should be placed at on creation.
#[derive(Default, Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub enum WindowPosition {
    /// Position will be set by the window manager
    #[default]
    Automatic,
    /// Window will be centered on the selected monitor
    ///
    /// Note that this does not account for window decorations.
    Centered(MonitorSelection),
    /// The window's top-left corner will be placed at the specified position (in pixels)
    ///
    /// (0,0) represents top-left corner of screen space.
    At(IVec2),
}

impl WindowPosition {
    pub fn new(position: IVec2) -> Self {
        Self::At(position)
    }

    pub fn set(&mut self, position: IVec2) {
        *self = WindowPosition::At(position);
    }
}

/// ## Window Sizes
///
/// There are three sizes associated with a window. The physical size which is
/// the height and width in physical pixels on the monitor. The logical size
/// which is the physical size scaled by an operating system provided factor to
/// account for monitors with differing pixel densities or user preference. And
/// the requested size, measured in logical pixels, which is the value submitted
/// to the API when creating the window, or requesting that it be resized.
///
/// The actual size, in logical pixels, of the window may not match the
/// requested size due to operating system limits on the window size, or the
/// quantization of the logical size when converting the physical size to the
/// logical size through the scaling factor.
// TODO: Make sure this is used correctly
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct WindowResolution {
    requested_width: f64,
    requested_height: f64,
    physical_width: u32,
    physical_height: u32,
    scale_factor_override: Option<f64>,
    scale_factor: f64,
}

impl Default for WindowResolution {
    fn default() -> Self {
        WindowResolution {
            requested_width: 1280.,
            requested_height: 720.,
            physical_width: 1280,
            physical_height: 720,
            scale_factor_override: None,
            scale_factor: 1.0,
        }
    }
}

impl WindowResolution {
    pub fn new(requested_width: f64, requested_height: f64) -> Self {
        Self {
            requested_width,
            requested_height,
            physical_width: requested_width as u32,
            physical_height: requested_height as u32,
            ..Default::default()
        }
    }

    pub fn new_with_scale_factor_override(
        requested_width: f64,
        requested_height: f64,
        scale_factor_override: f64,
    ) -> Self {
        Self {
            requested_width,
            requested_height,
            physical_width: requested_width as u32,
            physical_height: requested_height as u32,
            scale_factor_override: Some(scale_factor_override),
            ..Default::default()
        }
    }

    /// The current requested width of the window's client area.
    #[inline]
    pub fn requested_width(&self) -> f64 {
        self.requested_width
    }

    /// The current requested height of the window's client area.
    #[inline]
    pub fn requested_height(&self) -> f64 {
        self.requested_height
    }

    /// The window's client area width in logical pixels.
    #[inline]
    pub fn width(&self) -> f64 {
        self.physical_width() as f64 / self.scale_factor()
    }

    /// The window's client area width in logical pixels.
    #[inline]
    pub fn height(&self) -> f64 {
        self.physical_height() as f64 / self.scale_factor()
    }

    /// The window's client area width in physical pixels.
    #[inline]
    pub fn physical_width(&self) -> u32 {
        self.physical_width
    }

    /// The window's client area height in physical pixels.
    #[inline]
    pub fn physical_height(&self) -> u32 {
        self.physical_height
    }

    /// The ratio of physical pixels to logical pixels
    ///
    /// `physical_pixels = logical_pixels * scale_factor`
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor_override
            .unwrap_or(self.base_scale_factor())
    }

    /// The window scale factor as reported by the window backend.
    ///
    /// This value is unaffected by [`scale_factor_override`](Window::scale_factor_override).
    #[inline]
    pub fn base_scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// The scale factor set with [`set_scale_factor_override`](Window::set_scale_factor_override).
    ///
    /// This value may be different from the scale factor reported by the window backend.
    #[inline]
    pub fn scale_factor_override(&self) -> Option<f64> {
        self.scale_factor_override
    }

    /// Set the window's requested resolution.
    #[inline]
    pub fn set_requested_resolution(&mut self, width: f64, height: f64) {
        self.requested_width = width;
        self.requested_height = height;
    }

    /// Set the window's physical resolution.
    ///
    /// You probably don't want to call this directly unless you are dealing
    /// with a window manager library.
    #[inline]
    pub fn set_physical_resolution(&mut self, width: u32, height: u32) {
        self.physical_width = width;
        self.physical_height = height;
    }

    /// Set the window's scale factor, this may get overriden by the backend.
    #[inline]
    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    /// Set the window's scale factor, this will be used over what the backend decides.
    #[inline]
    pub fn set_scale_factor_override(&mut self, scale_factor_override: Option<f64>) {
        self.scale_factor_override = scale_factor_override;
    }
}

impl<I> From<(I, I)> for WindowResolution
where
    I: Into<f64>,
{
    fn from((width, height): (I, I)) -> WindowResolution {
        WindowResolution::new(width.into(), height.into())
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct WindowTitle {
    title: Cow<'static, str>,
}

impl Default for WindowTitle {
    fn default() -> Self {
        WindowTitle::new("Bevy App")
    }
}

impl WindowTitle {
    /// Creates a new [`WindowTitle`] from any string-like type.
    pub fn new(title: impl Into<Cow<'static, str>>) -> Self {
        WindowTitle {
            title: title.into(),
        }
    }

    /// Sets the window's title.
    #[inline(always)]
    pub fn set(&mut self, title: impl Into<Cow<'static, str>>) {
        *self = WindowTitle::new(title.into());
    }

    /// Gets the title of the window as a `&str`.
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.title
    }
}

impl<I> From<I> for WindowTitle
where
    I: Into<Cow<'static, str>>,
{
    fn from(title: I) -> WindowTitle {
        WindowTitle::new(title)
    }
}

#[derive(Default, Component, Debug, Copy, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum WindowDecorations {
    /// Window will have decorations (title, border, etc.)
    #[default]
    Decorated,

    /// Window will not have decorations
    Undecorated,
}

impl WindowDecorations {
    pub fn decorated(&self) -> bool {
        *self == Self::Decorated
    }
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct WindowFocus(bool);

impl Default for WindowFocus {
    fn default() -> Self {
        WindowFocus(false) // more explicitly we aren't focused by default
    }
}

impl WindowFocus {
    pub fn focused(&self) -> bool {
        self.0
    }

    pub fn set(&mut self, focused: bool) {
        self.0 = focused;
    }
}

#[derive(Default, Component, Debug, Copy, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum WindowResizable {
    /// This window is allowed to be resized by the user.
    #[default]
    Resizable,

    /// This window is not allowed to be resized by the user.
    ///
    /// Note: This does not stop the program from fullscreening/setting
    /// the size programmatically.
    Unresizable,
}

impl WindowResizable {
    pub fn resizable(&self) -> bool {
        *self == Self::Resizable
    }
}

#[derive(Default, Component, Debug, Copy, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum WindowTransparency {
    /// The window will have an opaque background by default.
    #[default]
    Opaque,

    /// The window's background will be see-through/transparent.
    Transparent,
}

impl WindowTransparency {
    pub fn transparent(&self) -> bool {
        *self == Self::Transparent
    }
}

#[derive(Default, Component, Debug, Copy, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum WindowState {
    /// The window is floating, this mostly just means that it is
    /// neither maximized nor minimized.
    #[default]
    Floating,

    /// The window is minimized to the task bar, but the program is
    /// still running.
    Minimized,

    /// The window is taking up the maximum amount of space on the
    /// window it is allowed to, without becoming fullscreen.
    Maximized,
}

#[derive(Default, Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct WindowCanvas {
    canvas: Option<String>,
    fit_canvas_to_parent: bool,
}

impl WindowCanvas {
    pub fn new(canvas: Option<String>, fit_canvas_to_parent: bool) -> Self {
        Self {
            canvas,
            fit_canvas_to_parent,
        }
    }

    /// The "html canvas" element selector. If set, this selector will be used to find a matching html canvas element,
    /// rather than creating a new one.   
    /// Uses the [CSS selector format](https://developer.mozilla.org/en-US/docs/Web/API/Document/querySelector).
    ///
    /// This value has no effect on non-web platforms.
    #[inline]
    pub fn canvas(&self) -> Option<&str> {
        self.canvas.as_deref()
    }

    /// Whether or not to fit the canvas element's size to its parent element's size.
    ///
    /// **Warning**: this will not behave as expected for parents that set their size according to the size of their
    /// children. This creates a "feedback loop" that will result in the canvas growing on each resize. When using this
    /// feature, ensure the parent's size is not affected by its children.
    ///
    /// This value has no effect on non-web platforms.
    #[inline]
    pub fn fit_canvas_to_parent(&self) -> bool {
        self.fit_canvas_to_parent
    }
}

/// Defines which monitor to use.
#[derive(Debug, Clone, Copy, Reflect)]
pub enum MonitorSelection {
    /// Uses current monitor of the window.
    Current,
    /// Uses primary monitor of the system.
    Primary,
    /// Uses monitor with the specified index.
    Number(usize),
}
