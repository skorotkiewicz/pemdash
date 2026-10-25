use crate::clipboard_helper::ClipboardHelper;
use crate::{AngleMode, calculate_with_angle_mode};

use gtk4::gdk;
use gtk4::gdk::prelude::ToplevelExt;
use gtk4::glib::Propagation;
use gtk4::prelude::{
    ApplicationExt, ApplicationExtManual, BoxExt, ButtonExt, Cast, EditableExt, EntryExt,
    EventControllerExt, GestureSingleExt, GtkApplicationExt, GtkWindowExt, NativeExt, ObjectExt,
    PopoverExt, WidgetExt,
};
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Entry,
    EventControllerKey, GestureClick, Label, MenuButton, Orientation, Popover, PropagationPhase,
    Switch,
};
const APP_ID: &str = "dev.pixelcluster.pemdash";

/// Runs the calculator overlay on the main thread until its window closes.
///
/// # Panics
///
/// Panics if GTK was initialized on a different thread.
///
/// # Examples
///
/// ```no_run
/// pemdash::run();
/// ```
pub fn run() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    if let Some(window) = app.active_window() {
        window.present();
        return;
    }

    load_css();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("pemdash")
        .default_width(720)
        .default_height(300)
        .decorated(false)
        .resizable(false)
        .build();
    window.add_css_class("pemdash");

    let input = Entry::builder()
        .name("expression")
        .placeholder_text("Try 50 / sqrt(3) * 1e5")
        .hexpand(true)
        .build();
    input.add_css_class("expression");
    input.set_primary_icon_name(Some("accessories-calculator-symbolic"));
    input.set_primary_icon_activatable(false);
    input.set_secondary_icon_tooltip_text(Some("Clear expression"));
    input.connect_icon_press(|entry, position| {
        if position == gtk4::EntryIconPosition::Secondary {
            entry.set_text("");
            entry.grab_focus();
        }
    });

    let mode = Switch::builder()
        .name("angle-mode")
        .valign(Align::Center)
        .tooltip_text("Angles use radians. Switch on to use degrees.")
        .build();
    mode.add_css_class("angle-switch");
    let radians = Label::new(Some("RAD"));
    radians.add_css_class("angle-unit");
    radians.add_css_class("selected");
    let degrees = Label::new(Some("DEG"));
    degrees.add_css_class("angle-unit");
    let angle_control = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .hexpand(true)
        .halign(Align::End)
        .build();
    angle_control.add_css_class("angle-control");
    angle_control.append(&radians);
    angle_control.append(&mode);
    angle_control.append(&degrees);

    let header = GtkBox::new(Orientation::Horizontal, 10);
    header.add_css_class("header");
    let drag_area = GtkBox::builder()
        .name("header-drag-area")
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .hexpand(true)
        .build();
    let mark = Label::new(Some("="));
    mark.add_css_class("brand-mark");
    let title = Label::new(Some("pemdash"));
    title.add_css_class("brand-name");
    let subtitle = Label::builder()
        .label("A little space to think.")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    subtitle.add_css_class("subtitle");
    let help = MenuButton::builder()
        .name("help")
        .icon_name("help-about-symbolic")
        .tooltip_text("Functions and examples")
        .popover(&reference_popover(&input, AngleMode::Radians))
        .build();
    help.add_css_class("quiet-button");
    let close = Button::builder()
        .name("close")
        .icon_name("window-close-symbolic")
        .tooltip_text("Close (Esc)")
        .build();
    close.add_css_class("quiet-button");
    {
        let window = window.downgrade();
        close.connect_clicked(move |_| {
            if let Some(window) = window.upgrade() {
                window.close();
            }
        });
    }
    drag_area.append(&mark);
    drag_area.append(&title);
    drag_area.append(&subtitle);
    header.append(&drag_area);
    header.append(&help);
    header.append(&close);

    let drag = GestureClick::new();
    drag.set_button(1);
    {
        let window = window.downgrade();
        drag.connect_pressed(move |gesture, _, x, y| {
            if let Some(window) = window.upgrade()
                && let Some(event) = gesture.current_event()
                && let Some(device) = event.device()
                && let Some(widget) = gesture.widget()
                && let Some(point) =
                    widget.compute_point(&window, &gtk4::graphene::Point::new(x as f32, y as f32))
                && let Some(surface) = window.surface()
                && let Ok(toplevel) = surface.downcast::<gdk::Toplevel>()
            {
                toplevel.begin_move(&device, 1, point.x() as f64, point.y() as f64, event.time());
            }
        });
    }
    drag_area.add_controller(drag);

    let expression_heading = Label::builder()
        .label("EXPRESSION")
        .halign(Align::Start)
        .build();
    expression_heading.add_css_class("eyebrow");
    let expression_section = GtkBox::new(Orientation::Vertical, 8);
    expression_section.append(&expression_heading);
    expression_section.append(&input);

    let result = Label::builder()
        .name("result")
        .label("Enter an expression")
        .halign(Align::Fill)
        .xalign(0.0)
        .hexpand(true)
        .selectable(true)
        .ellipsize(gtk4::pango::EllipsizeMode::Middle)
        .build();
    result.add_css_class("result");
    result.add_css_class("empty");

    let result_heading = Label::builder()
        .label("RESULT")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    result_heading.add_css_class("eyebrow");
    let status = Label::builder()
        .name("result-status")
        .label("Ready when you are")
        .build();
    status.add_css_class("status");
    let result_header = GtkBox::new(Orientation::Horizontal, 8);
    result_header.append(&result_heading);
    result_header.append(&status);

    let copy = Button::with_label("Copy");
    copy.set_widget_name("copy-result");
    copy.set_sensitive(false);
    copy.set_valign(Align::Center);
    copy.set_tooltip_text(Some("Copy result without closing"));
    copy.add_css_class("copy");

    let result_row = GtkBox::new(Orientation::Horizontal, 16);
    result_row.append(&result);
    result_row.append(&copy);
    let result_card = GtkBox::new(Orientation::Vertical, 12);
    result_card.add_css_class("result-card");
    result_card.append(&result_header);
    result_card.append(&result_row);

    let footer = GtkBox::new(Orientation::Horizontal, 6);
    footer.add_css_class("footer");
    footer.set_widget_name("keyboard-hint");
    footer.set_tooltip_text(Some("Enter: copy valid result and close   Esc: close"));
    for (key, action) in [("↵ Enter", "copy & close"), ("Esc", "close")] {
        let keycap = Label::new(Some(key));
        keycap.add_css_class("keycap");
        let description = Label::new(Some(action));
        description.add_css_class("shortcut-description");
        footer.append(&keycap);
        footer.append(&description);
    }
    footer.append(&angle_control);

    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(18)
        .build();
    content.add_css_class("content");
    content.append(&header);
    content.append(&expression_section);
    content.append(&result_card);
    content.append(&footer);
    window.set_child(Some(&content));

    {
        let result = result.clone();
        let copy = copy.clone();
        let status = status.clone();
        let result_card = result_card.clone();
        let mode = mode.clone();
        input.connect_changed(move |input| {
            refresh_result(
                input,
                &result,
                &copy,
                &status,
                &result_card,
                angle_mode(&mode),
            );
        });
    }

    {
        let input = input.downgrade();
        let help = help.downgrade();
        let result = result.clone();
        let copy = copy.clone();
        let status = status.clone();
        let result_card = result_card.clone();
        mode.connect_active_notify(move |mode| {
            let selected = angle_mode(mode);
            radians.remove_css_class("selected");
            degrees.remove_css_class("selected");
            let tooltip = match selected {
                AngleMode::Radians => {
                    radians.add_css_class("selected");
                    "Angles use radians. Switch on to use degrees."
                }
                AngleMode::Degrees => {
                    degrees.add_css_class("selected");
                    "Angles use degrees. Switch off to use radians."
                }
            };
            mode.set_tooltip_text(Some(tooltip));
            if let Some(input) = input.upgrade() {
                refresh_result(&input, &result, &copy, &status, &result_card, selected);
                if let Some(help) = help.upgrade() {
                    help.set_popover(Some(&reference_popover(&input, selected)));
                }
                input.grab_focus();
            }
        });
    }

    {
        let window = window.downgrade();
        let input = input.downgrade();
        let status = status.downgrade();
        let mode = mode.downgrade();
        copy.connect_clicked(move |copy| {
            if let (Some(window), Some(input), Some(status), Some(mode)) = (
                window.upgrade(),
                input.upgrade(),
                status.upgrade(),
                mode.upgrade(),
            ) {
                copy_result(
                    window.upcast(),
                    input,
                    copy.clone(),
                    status,
                    CopyAction::KeepOpen,
                    angle_mode(&mode),
                );
            }
        });
    }
    {
        let window = window.downgrade();
        let copy = copy.downgrade();
        let status = status.downgrade();
        let mode = mode.downgrade();
        input.connect_activate(move |input| {
            if let (Some(window), Some(copy), Some(status), Some(mode)) = (
                window.upgrade(),
                copy.upgrade(),
                status.upgrade(),
                mode.upgrade(),
            ) {
                copy_result(
                    window.upcast(),
                    input.clone(),
                    copy,
                    status,
                    CopyAction::Close,
                    angle_mode(&mode),
                );
            }
        });
    }

    let keys = EventControllerKey::new();
    keys.set_propagation_phase(PropagationPhase::Capture);
    {
        let window = window.downgrade();
        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                if let Some(window) = window.upgrade() {
                    window.close();
                }
                Propagation::Stop
            } else {
                Propagation::Proceed
            }
        });
    }
    window.add_controller(keys);

    window.present();
    input.grab_focus();
}

fn angle_mode(switch: &Switch) -> AngleMode {
    if switch.is_active() {
        AngleMode::Degrees
    } else {
        AngleMode::Radians
    }
}

fn refresh_result(
    input: &Entry,
    result: &Label,
    copy: &Button,
    status: &Label,
    result_card: &GtkBox,
    mode: AngleMode,
) {
    result.remove_css_class("error");
    result.remove_css_class("empty");
    result_card.remove_css_class("invalid");
    result_card.remove_css_class("valid");
    result.set_tooltip_text(None);
    copy.remove_css_class("copied");
    copy.set_label("Copy");
    status.remove_css_class("error");
    status.set_tooltip_text(None);

    let expression = input.text();
    input.set_secondary_icon_name((!expression.is_empty()).then_some("edit-clear-symbolic"));
    if expression.trim().is_empty() {
        result.set_text("Enter an expression");
        result.add_css_class("empty");
        status.set_text("Ready when you are");
        copy.set_sensitive(false);
        return;
    }

    match calculate_with_angle_mode(&expression, mode) {
        Ok(value) => {
            let text = format_result(value);
            result.set_text(&format!("= {text}"));
            result.set_tooltip_text(Some(&text));
            result_card.add_css_class("valid");
            status.set_text("Live result");
            copy.set_sensitive(true);
        }
        Err(error) => {
            result.set_text(&error);
            result.set_tooltip_text(Some(&error));
            result.add_css_class("error");
            result_card.add_css_class("invalid");
            status.set_text("Check expression");
            copy.set_sensitive(false);
        }
    }
}

enum CopyAction {
    KeepOpen,
    Close,
}

fn copy_result(
    window: gtk4::Window,
    input: Entry,
    copy: Button,
    status: Label,
    action: CopyAction,
    mode: AngleMode,
) {
    let Ok(value) = calculate_with_angle_mode(&input.text(), mode) else {
        return;
    };
    let Some(content) = window.child() else {
        return;
    };
    if !content.is_sensitive() {
        return;
    }

    let text = format_result(value);
    content.set_sensitive(false);
    copy.set_label("Copying…");
    copy.remove_css_class("copied");
    status.set_text("Copying…");
    status.remove_css_class("error");
    status.set_tooltip_text(None);
    gtk4::glib::spawn_future_local(async move {
        let display = WidgetExt::display(&window);
        let outcome = match ClipboardHelper::for_backend(display.type_().name()) {
            Ok(helper) => helper.copy_text(&text).await,
            Err(error) => Err(error),
        };
        content.set_sensitive(true);
        match outcome {
            Ok(()) => {
                copy.set_label("Copied");
                copy.add_css_class("copied");
                status.set_text("Copied to clipboard");
                if matches!(action, CopyAction::Close) {
                    window.close();
                }
            }
            Err(error) => {
                copy.set_label("Retry");
                status.set_text("Copy failed");
                status.add_css_class("error");
                status.set_tooltip_text(Some(&error));
                input.grab_focus();
            }
        }
    });
}

fn reference_popover(input: &Entry, mode: AngleMode) -> Popover {
    let popover = Popover::new();
    popover.add_css_class("reference-popover");
    let content = GtkBox::new(Orientation::Vertical, 12);
    content.add_css_class("reference-content");
    let title = Label::builder()
        .label("A few things you can do")
        .halign(Align::Start)
        .build();
    title.add_css_class("reference-title");
    content.append(&title);

    for (heading, detail) in [
        ("Operators", "+  −  *  /  ^  ( )"),
        ("Constants", "pi, e, tau · explicit multiplication: 2 * pi"),
        ("Functions", "sqrt, abs, exp, ln, log10, min, max, pow"),
        (
            "Trigonometry",
            "sin, cos, tan, asin, acos, atan, atan2 · use RAD / DEG",
        ),
        ("Rounding", "floor, ceil, round"),
    ] {
        let row = GtkBox::new(Orientation::Vertical, 3);
        let heading = Label::builder().label(heading).halign(Align::Start).build();
        heading.add_css_class("reference-heading");
        let detail = Label::builder().label(detail).halign(Align::Start).build();
        detail.add_css_class("reference-detail");
        row.append(&heading);
        row.append(&detail);
        content.append(&row);
    }

    let examples_heading = Label::builder()
        .label("TRY AN EXAMPLE")
        .halign(Align::Start)
        .build();
    examples_heading.add_css_class("eyebrow");
    content.append(&examples_heading);
    let examples = GtkBox::new(Orientation::Horizontal, 6);
    let trig_example = match mode {
        AngleMode::Radians => "sin(pi / 2)",
        AngleMode::Degrees => "sin(90)",
    };
    for expression in ["24 * 1.25", "sqrt(144)", trig_example] {
        let button = Button::with_label(expression);
        button.add_css_class("example");
        let input = input.downgrade();
        let popover = popover.downgrade();
        button.connect_clicked(move |_| {
            if let Some(popover) = popover.upgrade() {
                popover.popdown();
            }
            if let Some(input) = input.upgrade() {
                input.set_text(expression);
                input.grab_focus();
                input.set_position(-1);
            }
        });
        examples.append(&button);
    }
    content.append(&examples);
    popover.set_child(Some(&content));
    popover
}

fn format_result(value: f64) -> String {
    if value == 0.0 {
        "0".into()
    } else {
        value.to_string()
    }
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk4::prelude::{IsA, ListModelExt};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn find_widget<T: IsA<gtk4::Widget> + Clone + 'static>(
        root: &impl IsA<gtk4::Widget>,
        name: &str,
    ) -> T {
        fn find(root: &gtk4::Widget, name: &str) -> Option<gtk4::Widget> {
            if root.widget_name() == name {
                return Some(root.clone());
            }
            let mut child = root.first_child();
            while let Some(widget) = child {
                if let Some(found) = find(&widget, name) {
                    return Some(found);
                }
                child = widget.next_sibling();
            }
            None
        }
        find(root.as_ref(), name)
            .unwrap_or_else(|| panic!("Missing widget: {name}"))
            .downcast::<T>()
            .unwrap_or_else(|_| panic!("Wrong widget type: {name}"))
    }

    #[test]
    #[ignore = "requires a display and clipboard helpers; run cargo test --lib -- --ignored"]
    fn ui_clears_stale_results_and_handles_keyboard_actions() {
        gtk4::init().expect("GTK needs a graphical display");
        let app = Application::builder()
            .application_id("dev.pixelcluster.pemdash.Tests")
            .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
            .build();
        app.register(None::<&gtk4::gio::Cancellable>).unwrap();

        let css_errors = Rc::new(RefCell::new(Vec::new()));
        let provider = CssProvider::new();
        {
            let css_errors = css_errors.clone();
            provider.connect_parsing_error(move |_, _, error| {
                css_errors.borrow_mut().push(error.to_string());
            });
        }
        provider.load_from_data(include_str!("style.css"));
        assert!(css_errors.borrow().is_empty(), "{:?}", css_errors.borrow());

        let widgets = || {
            build_ui(&app);
            let window = app.active_window().unwrap();
            let input = find_widget::<Entry>(&window, "expression");
            let result = find_widget::<Label>(&window, "result");
            let copy = find_widget::<Button>(&window, "copy-result");
            let status = find_widget::<Label>(&window, "result-status");
            (window, input, result, copy, status)
        };

        let clipboard_text = || {
            let display = gdk::Display::default().unwrap();
            let command: &[&str] = match display.type_().name() {
                "GdkWaylandDisplay" => &["wl-paste", "--no-newline"],
                "GdkX11Display" => &["xclip", "-selection", "clipboard", "-out"],
                "GdkMacosDisplay" => &["/usr/bin/pbpaste"],
                "GdkWin32Display" => &[
                    "powershell.exe",
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-STA",
                    "-Command",
                    "$ErrorActionPreference = 'Stop'; \
                     [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); \
                     [Console]::Write((Get-Clipboard -Raw))",
                ],
                _ => panic!("Clipboard test requires Linux, macOS, or Windows"),
            };
            // An external reader also works when the test window has no
            // keyboard focus (required for GTK clipboard reads on Wayland).
            let output = std::process::Command::new(command[0])
                .args(&command[1..])
                .output()
                .unwrap();
            assert!(output.status.success(), "{:?}", output);
            Some(String::from_utf8(output.stdout).unwrap())
        };
        let wait_for_copy = |window: &gtk4::Window| {
            let content = window.child().unwrap();
            gtk4::glib::MainContext::default().block_on(async {
                gtk4::glib::future_with_timeout(std::time::Duration::from_secs(6), async {
                    while !content.is_sensitive() {
                        gtk4::glib::timeout_future(std::time::Duration::from_millis(10)).await;
                    }
                })
                .await
                .expect("Copy operation did not complete");
            });
        };
        let (window, input, result, copy, status) = widgets();
        let drag_area = find_widget::<GtkBox>(&window, "header-drag-area");
        let help = find_widget::<MenuButton>(&window, "help");
        let close = find_widget::<Button>(&window, "close");
        assert_eq!(drag_area.parent(), help.parent());
        assert_eq!(drag_area.parent(), close.parent());
        let controllers = drag_area.observe_controllers();
        assert!((0..controllers.n_items()).any(|i| {
            controllers
                .item(i)
                .is_some_and(|item| item.is::<GestureClick>())
        }));
        let hint = find_widget::<GtkBox>(&window, "keyboard-hint");
        assert_eq!(
            hint.tooltip_text().as_deref(),
            Some("Enter: copy valid result and close   Esc: close")
        );
        assert_eq!(status.text(), "Ready when you are");
        assert!(result.has_css_class("empty"));
        assert!(!copy.is_sensitive());
        input.emit_activate();
        assert!(window.is_visible());

        let mode = find_widget::<Switch>(&window, "angle-mode");
        let radians = mode.prev_sibling().expect("Switch has a RAD label");
        let degrees = mode.next_sibling().expect("Switch has a DEG label");
        assert!(radians.has_css_class("selected"), "RAD starts selected");
        assert!(!mode.is_active());
        input.set_text("sin(90)");
        let radians_result = result.text();
        mode.set_active(true);
        assert!(degrees.has_css_class("selected"), "DEG becomes selected");
        assert!(!radians.has_css_class("selected"), "RAD becomes inactive");
        assert_eq!(input.text(), "sin(90)");
        assert_eq!(result.text(), "= 1");
        copy.emit_clicked();
        wait_for_copy(&window);
        assert_eq!(clipboard_text().as_deref(), Some("1"));
        assert!(window.is_visible());
        let reference = help.popover().expect("Help has a reference popover");
        let trig_example = reference
            .child()
            .and_then(|content| content.last_child())
            .and_then(|examples| examples.last_child())
            .expect("Reference includes a trigonometry example")
            .downcast::<Button>()
            .expect("Example is a button");
        assert_eq!(trig_example.label().as_deref(), Some("sin(90)"));
        mode.set_active(false);
        assert!(radians.has_css_class("selected"), "RAD becomes selected");
        assert!(!degrees.has_css_class("selected"), "DEG becomes inactive");
        assert_eq!(result.text(), radians_result);
        assert_eq!(copy.label().as_deref(), Some("Copy"));
        assert!(!copy.has_css_class("copied"));

        input.set_text("2 + 2");
        assert_eq!(result.text(), "= 4");
        assert_eq!(status.text(), "Live result");
        assert!(!result.has_css_class("empty"));
        assert!(copy.is_sensitive());
        copy.emit_clicked();
        assert_eq!(copy.label().as_deref(), Some("Copying…"));
        input.emit_activate();
        assert!(window.is_visible());
        wait_for_copy(&window);
        assert_eq!(copy.label().as_deref(), Some("Copied"));
        assert!(copy.has_css_class("copied"));
        assert_eq!(status.text(), "Copied to clipboard");
        assert_eq!(clipboard_text().as_deref(), Some("4"));
        assert!(window.is_visible());

        for expression in ["1 / 0", "sqrt(-1)", "1e999", "2 +", ""] {
            input.set_text(expression);
            assert!(!copy.is_sensitive(), "{expression}");
            assert_eq!(copy.label().as_deref(), Some("Copy"));
            input.emit_activate();
            assert!(window.is_visible(), "{expression}");
        }
        assert_eq!(clipboard_text().as_deref(), Some("4"));
        assert_eq!(result.text(), "Enter an expression");
        assert!(result.has_css_class("empty"));
        assert!(!result.has_css_class("error"));
        assert!(!copy.has_css_class("copied"));
        assert_eq!(status.text(), "Ready when you are");

        let reference = help.popover().unwrap();
        let examples = reference.child().unwrap().last_child().unwrap();
        let example = examples
            .first_child()
            .unwrap()
            .downcast::<Button>()
            .unwrap();
        example.emit_clicked();
        assert_eq!(input.text(), "24 * 1.25");
        assert_eq!(result.text(), "= 30");
        input.emit_by_name::<()>("icon-press", &[&gtk4::EntryIconPosition::Secondary]);
        assert_eq!(input.text(), "");
        assert!(!copy.is_sensitive());

        input.set_text("1e300");
        assert_eq!(
            result.tooltip_text().as_deref(),
            Some(format_result(1e300).as_str())
        );
        assert_eq!(result.ellipsize(), gtk4::pango::EllipsizeMode::Middle);

        input.set_text("sqrt(-1)");
        assert!(result.has_css_class("error"));
        assert_eq!(status.text(), "Check expression");
        input.set_text("2^3^2");
        assert_eq!(result.text(), "= 512");
        assert!(!result.has_css_class("error"));
        assert!(copy.is_sensitive());
        input.emit_activate();
        assert!(window.is_visible());
        wait_for_copy(&window);
        assert!(!window.is_visible());
        assert_eq!(clipboard_text().as_deref(), Some("512"));

        let (window, input, _, copy, _) = widgets();
        input.set_text("1");
        copy.grab_focus();
        let controllers = window.observe_controllers();
        let keys = (0..controllers.n_items())
            .filter_map(|i| controllers.item(i)?.downcast::<EventControllerKey>().ok())
            .next()
            .unwrap();
        assert_eq!(keys.propagation_phase(), PropagationPhase::Capture);
        let emit_key = |key: gdk::Key| {
            keys.emit_by_name::<bool>("key-pressed", &[&key, &0_u32, &gdk::ModifierType::empty()])
        };
        assert!(!emit_key(gdk::Key::a));
        assert!(window.is_visible());
        assert!(emit_key(gdk::Key::Escape));
        assert!(!window.is_visible());

        let (window, input, result, _, _) = widgets();
        let mode = find_widget::<Switch>(&window, "angle-mode");
        assert!(!mode.is_active(), "New windows default to radians");
        mode.set_active(true);
        assert_eq!(result.text(), "Enter an expression");
        input.set_text("asin(1)");
        assert_eq!(result.text(), "= 90");
        input.emit_activate();
        wait_for_copy(&window);
        assert!(!window.is_visible());
        assert_eq!(clipboard_text().as_deref(), Some("90"));

        let (window, _, _, _, _) = widgets();
        find_widget::<Button>(&window, "close").emit_clicked();
        assert!(!window.is_visible());
    }

    #[test]
    fn formats_whole_numbers_without_decimal_noise() {
        assert_eq!(format_result(6.0), "6");
        assert_eq!(format_result(-0.0), "0");
    }
}
