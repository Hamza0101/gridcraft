//! Space on a checkbox cell toggles it and nothing else (#1951).

use egui::{Event, Key, Modifiers};
use gridcraft_engine::Session;
use gridcraft_engine::core::CellRef;
use gridcraft_engine::core::Value;
use gridcraft_ui_egui::SheetApp;
use serde_json::json;

type Harness = egui_kittest::Harness<'static, SheetApp>;

fn harness(session: Session) -> Harness {
    let mut h = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        SheetApp::new(session, Default::default()),
    );
    h.run_steps(4);
    h
}

/// A space, as a keyboard reports it: the key press, the text it produces, then the release.
fn space(h: &mut Harness) {
    let k = |pressed| Event::Key { key: Key::Space, physical_key: Some(Key::Space), pressed, repeat: false, modifiers: Modifiers::NONE };
    h.input_mut().events.push(k(true));
    h.input_mut().events.push(Event::Text(" ".into()));
    h.input_mut().events.push(k(false));
    h.run_steps(2);
}

fn press(h: &mut Harness, key: Key) {
    let k = |pressed| Event::Key { key, physical_key: Some(key), pressed, repeat: false, modifiers: Modifiers::NONE };
    h.input_mut().events.push(k(true));
    h.input_mut().events.push(k(false));
    h.run_steps(2);
}

fn value(h: &Harness, a1: &str) -> Option<Value> {
    let d = h.state().session.active().unwrap();
    d.wb.active().unwrap().cell(CellRef::parse(a1).unwrap()).map(|c| c.value.clone())
}

#[test]
fn space_toggles_a_checkbox_cell_without_editing_it() {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "B2", "input": "FALSE"})).unwrap();
    s.execute("insert.checkbox", json!({"range": "B2"})).unwrap();
    s.execute("selection.set", json!({"range": "B2"})).unwrap();
    let mut h = harness(s);
    assert_eq!(value(&h, "B2"), Some(Value::Bool(false)));

    // The key press toggles the box; the text it also produces must not start editing, or Enter
    // commits that space over the checkbox and leaves the cell blank.
    space(&mut h);
    assert_eq!(value(&h, "B2"), Some(Value::Bool(true)));
    assert!(h.state().editor.is_none(), "no editor opened");
    press(&mut h, Key::Enter);
    assert_eq!(value(&h, "B2"), Some(Value::Bool(true)), "Enter moved on and left the checkbox");

    // And again, back to unchecked.
    s_select(&mut h, "B2");
    space(&mut h);
    assert_eq!(value(&h, "B2"), Some(Value::Bool(false)));
    assert!(h.state().editor.is_none());

    // A space still types on an ordinary cell.
    s_select(&mut h, "C3");
    space(&mut h);
    assert!(h.state().editor.is_some(), "an ordinary cell starts editing");
    press(&mut h, Key::Escape);
}

fn s_select(h: &mut Harness, range: &str) {
    h.state_mut().session.execute("selection.set", json!({"range": range})).unwrap();
    h.run_steps(2);
}
