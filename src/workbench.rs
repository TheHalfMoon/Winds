use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Paragraph};

const EMPTY_WORKBENCH_MESSAGE: &str = "No terminal panes are active.";

/// Render the inert T087 workbench shell without owning terminal runtime state.
pub(crate) fn render_inert_workbench(frame: &mut Frame<'_>) {
    let area = frame.area();
    let block = Block::bordered().title(" Winds Workbench ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width > 0 && inner.height > 0 {
        frame.render_widget(
            Paragraph::new(EMPTY_WORKBENCH_MESSAGE).alignment(Alignment::Center),
            inner,
        );
    }
}
