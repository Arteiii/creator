use crate::creator::{self, Creator};
use crate::directory_analyzer::DirectoryAnalyzer;
use crate::environment;
use cursive::align::HAlign;
use cursive::event::Key;
use cursive::views::{Dialog, EditView, LinearLayout, OnEventView, SelectView, TextView};
use cursive::Cursive;
use cursive::{traits::*, CursiveRunnable};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

const SELECT_GROUP_MSG: &str = "Select template group";
const SELECT_ITEM_MSG: &str = "Select template";

/// Run the tui application
pub fn run() {
    let mut tui = cursive::CursiveRunnable::default();
    tui.add_global_callback('q', |s| s.quit());
    show_main_screen(&mut tui);
    tui.run();
}

/// Shows Template type selection dialog
fn show_main_screen(cursive: &mut CursiveRunnable) {
    let template_storage_path = environment::get_storage_path();
    let mut select = build_select_view(&template_storage_path);

    select.set_on_submit(move |cursive_inst: &mut Cursive, selected: &str| {
        let group_path = format!("{}/{}", template_storage_path, selected);
        show_template_select(cursive_inst, group_path);
    });

    let sel_events = OnEventView::new(select)
        .on_event(Key::Esc, |cursive| cursive.quit())
        .scrollable()
        .fixed_size((20, 10));
    cursive.add_layer(Dialog::around(sel_events).title(SELECT_GROUP_MSG));
}

/// Shows template selection dialog
fn show_template_select(cursive: &mut Cursive, group_full_path: String) {
    // Create select
    let mut select = build_select_view(&group_full_path);
    select.set_on_submit(move |cursive_inst, selected_template: &str| {
        let template_full_path = format!("{}/{}", group_full_path, selected_template);
        show_variable_input_form(cursive_inst, template_full_path);
    });

    // Build the dialog
    let sel_events = OnEventView::new(select)
        .on_event(Key::Esc, |cursive_inst| {
            cursive_inst.pop_layer();
        })
        .scrollable()
        .fixed_size((20, 10));
    let dialog = Dialog::around(sel_events).title(SELECT_ITEM_MSG);
    cursive.add_layer(dialog);
}

/// Scanns the variable in the folder and asks for the user unput
fn show_variable_input_form(cursive: &mut Cursive, template_full_path: String) {
    let destination = environment::get_current_working_directory();
    let d_analyzer = DirectoryAnalyzer::new(&template_full_path);
    let variable_names = d_analyzer.scan_variables();

    // Create a vertical layout to hold input fields
    let mut layout = LinearLayout::vertical();
    for var in &variable_names {
        // Add a TextView and an EditView for each variable
        layout.add_child(TextView::new(format!("{}:", var)));
        layout.add_child(EditView::new().with_name(var.clone()));
    }

    // Wrap the layout in a Dialog with a submit button
    let dialog_title = format!("Enter Details \n Template: {}", template_full_path);
    let dialog = Dialog::around(layout.scrollable())
        .title(dialog_title)
        // Close the dialog on cancel
        .button("Back", move |cursive| {
            cursive.pop_layer();
        })
        // Copy the template on Create
        .button("Create", move |cursive| {
            create_from_template(
                cursive,
                &template_full_path,
                &destination,
                &variable_names,
            );
        });

    let dialog_ev = OnEventView::new(dialog).on_event(Key::Esc, move |cursive| {
        cursive.pop_layer();
    });

    cursive.add_layer(dialog_ev);
}

fn create_from_template(
    cursive: &mut Cursive,
    srs: &str,
    dest: &str,
    var_names: &HashSet<String>,
) {
    // Collect input values
    let mut input_values: HashMap<String, String> = HashMap::new();
    for var in var_names {
        let value = cursive
        .call_on_name(&var, |view: &mut EditView| view.get_content())
        .unwrap_or_default();
    input_values.insert(var.clone(), value.to_string());
}
    let mut creator = Creator::new(Path::new(srs), Path::new(dest));
    creator.set_var_values(&input_values);
    creator.create();
    
    let mut results = String::new();
    let src = creator.get_source().to_str().unwrap();
    let dsc = creator.get_destination().to_str().unwrap();
    results.push_str(&format!("Source: {}\n", src));
    results.push_str(&format!("Destination: {}\n", dsc));

    for (k, v) in creator.get_var_values() {
        results.push_str(&format!("{}: {}\n", k, v));
    }
    // Show results in a new dialog
    cursive.add_layer(Dialog::info(results));
    exit_after(cursive, 3);
}

/// Return a SelectView constructed from the folder names in the provided path
fn build_select_view(dir: &str) -> SelectView {
    let mut select = SelectView::new()
        // Center the text horizontally
        .h_align(HAlign::Center)
        // Use keyboard to jump to the pressed letters
        .autojump();

    // List the dirs
    let templ_dir = DirectoryAnalyzer::new(dir);
    let (_, directs) = templ_dir.get_items();

    let mut str_paths: Vec<String> = Vec::new();
    for d in directs {
        let s = d.file_name().unwrap().to_str().unwrap().to_string();
        str_paths.push(s);
    }
    select.add_all_str(str_paths);
    select
}

fn exit_after(siv: &mut Cursive, sec: u64) {
    let duration = Duration::from_secs(sec);
    let quit_callback = siv.cb_sink().clone();

    std::thread::spawn(move || {
        std::thread::sleep(duration);
        quit_callback
            .send(Box::new(|s: &mut Cursive| s.quit()))
            .unwrap();
    });
}
