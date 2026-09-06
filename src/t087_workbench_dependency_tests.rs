use crate::workbench::render_inert_workbench;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[test]
fn t087_inert_workbench_renders_deterministically_without_terminal_child() {
    let backend = TestBackend::new(48, 5);
    let mut terminal = Terminal::new(backend).expect("test terminal");

    terminal
        .draw(render_inert_workbench)
        .expect("first inert workbench render");
    let first = terminal.backend().buffer().clone();

    assert_eq!(first[(0, 0)].symbol(), "┌");
    assert_eq!(first[(47, 0)].symbol(), "┐");
    assert_eq!(first[(0, 4)].symbol(), "└");
    assert_eq!(first[(47, 4)].symbol(), "┘");
    let rendered_row = (0u16..48u16)
        .map(|x| first[(x, 1)].symbol())
        .collect::<Vec<_>>()
        .concat();
    assert!(
        rendered_row.contains("No terminal panes are active."),
        "expected empty-state message in rendered row: {rendered_row:?}"
    );

    terminal
        .draw(render_inert_workbench)
        .expect("second inert workbench render");

    assert_eq!(terminal.backend().buffer(), &first);
}

#[test]
fn t087_inert_workbench_handles_tiny_areas_without_runtime_side_effects() {
    for (width, height) in [(0, 0), (1, 1), (2, 2), (8, 3)] {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(render_inert_workbench)
            .expect("tiny inert workbench render");
    }
}
