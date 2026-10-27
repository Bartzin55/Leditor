//! A bounded-width overview. GTK owns text layout and cursor navigation.
use crate::editor_view::NumberedView;
use gtk::prelude::*;
use std::{cell::RefCell, rc::Rc};
const COLUMNS: usize = 240;
fn rows(text: &str) -> Vec<[u64; 4]> {
    text.split('\n')
        .map(|line| {
            let mut bits = [0; 4];
            let mut column = 0;
            for ch in line.chars() {
                if column >= COLUMNS {
                    break;
                }
                if !ch.is_whitespace() {
                    bits[column / 64] |= 1 << (column % 64);
                }
                column += if ch == '\t' { 4 - column % 4 } else { 1 };
            }
            bits
        })
        .collect()
}
pub fn create(view: &NumberedView, scroll: &gtk::ScrolledWindow) -> gtk::DrawingArea {
    let map = gtk::DrawingArea::builder()
        .width_request(120)
        .vexpand(true)
        .build();
    map.add_css_class("editor-surface");
    map.add_css_class("minimap");
    map.set_cursor_from_name(Some("pointer"));
    let buffer = view.buffer();
    let cache = Rc::new(RefCell::new(rows(&buffer.text(
        &buffer.start_iter(),
        &buffer.end_iter(),
        true,
    ))));
    let weak = map.downgrade();
    let data = cache.clone();
    buffer.connect_changed(move |buffer| {
        *data.borrow_mut() = rows(&buffer.text(&buffer.start_iter(), &buffer.end_iter(), true));
        if let Some(map) = weak.upgrade() {
            map.queue_draw();
        }
    });
    let weak_view = view.downgrade();
    map.set_draw_func(move |_, cr, width, height| {
        let Some(view) = weak_view.upgrade() else {
            return;
        };
        let rows = cache.borrow();
        let step = (height as f64 / rows.len().max(1) as f64).min(3.0);
        let color = view.color();
        cr.set_source_rgba(
            color.red() as f64,
            color.green() as f64,
            color.blue() as f64,
            0.65,
        );
        let pixels = (rows.len() as f64 * step).ceil() as usize;
        // At most one sampled logical line per vertical pixel for large files.
        for y in (0..pixels).step_by(step.ceil().max(1.0) as usize) {
            let line = ((y as f64 / step) as usize).min(rows.len() - 1);
            for col in 0..COLUMNS {
                if rows[line][col / 64] & (1 << (col % 64)) != 0 {
                    cr.rectangle(
                        col as f64 * width as f64 / COLUMNS as f64,
                        y as f64,
                        (width as f64 / COLUMNS as f64).max(0.5),
                        step.min(1.5),
                    );
                }
            }
        }
        let _ = cr.fill();
        let visible = view.visible_rect();
        let (first, _) = view.line_at_y(visible.y());
        let (last, _) = view.line_at_y(visible.y() + visible.height());
        cr.set_source_rgba(0.35, 0.55, 0.85, 0.24);
        cr.rectangle(
            0.0,
            first.line() as f64 * step,
            width as f64,
            ((last.line() - first.line() + 1) as f64 * step).max(2.0),
        );
        let _ = cr.fill();
    });
    let click = gtk::GestureClick::new();
    click.set_button(1);
    let weak_view = view.downgrade();
    let weak_map = map.downgrade();
    click.connect_pressed(move |_, _, _, y| {
        if let (Some(view), Some(map)) = (weak_view.upgrade(), weak_map.upgrade()) {
            let buffer = view.buffer();
            let step = (map.height().max(1) as f64 / buffer.line_count().max(1) as f64).min(3.0);
            let line = ((y / step) as i32).clamp(0, buffer.line_count() - 1);
            if let Some(mut iter) = buffer.iter_at_line(line) {
                buffer.place_cursor(&iter);
                view.scroll_to_iter(&mut iter, 0.0, true, 0.0, 0.5);
                view.grab_focus();
            }
        }
    });
    map.add_controller(click);
    for adjustment in [scroll.vadjustment(), scroll.hadjustment()] {
        let weak = map.downgrade();
        adjustment.connect_changed(move |_| {
            if let Some(map) = weak.upgrade() {
                map.queue_draw();
            }
        });
        let weak = map.downgrade();
        adjustment.connect_value_changed(move |_| {
            if let Some(map) = weak.upgrade() {
                map.queue_draw();
            }
        });
    }
    map
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overview_preserves_empty_lines_and_bounds_width() {
        let data = rows("a\n\n\tb\n");
        assert_eq!(data.len(), 4);
        assert_eq!(data[0][0], 1);
        assert_eq!(data[1], [0; 4]);
        assert_eq!(data[2][0], 1 << 4);
        assert_eq!(rows(&"x".repeat(10000))[0][3], (1 << 48) - 1);
    }
}
