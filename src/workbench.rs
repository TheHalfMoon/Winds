use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Widget},
};

const WORKBENCH_TITLE: &str = " Winds Workbench ";

/// An inert presentation-only workbench shell.
///
/// T087 deliberately owns no terminal child, event loop, parser, editor, history,
/// repository mutation, or provider/model authority. Later dependency-ordered
/// tasks may add those exact seams without turning presentation state into
/// lifecycle or verification authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InertWorkbench {
    width: u16,
    height: u16,
}

impl InertWorkbench {
    pub(crate) const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub(crate) const fn area(self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    pub(crate) fn render(self) -> Buffer {
        let area = self.area();
        let mut buffer = Buffer::empty(area);
        Block::bordered().title(WORKBENCH_TITLE).render(area, &mut buffer);
        buffer
    }
}

/// Proves that the selected Ratatui backend resolves to Crossterm without
/// entering raw mode or touching host terminal state.
pub(crate) fn backend_contract_name() -> &'static str {
    std::any::type_name::<ratatui::backend::CrosstermBackend<std::io::Stdout>>()
}

/// Proves that the explicitly selected Crossterm events feature is available
/// without reading an event or starting a persistent event loop.
pub(crate) fn event_contract_name() -> &'static str {
    std::any::type_name::<crossterm::event::Event>()
}
