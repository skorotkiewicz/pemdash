use gtk4::gdk;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Entry,
    EventControllerKey, Label, Orientation,
};
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "dev.pixelcluster.pemdash";

fn main() {
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
        .default_height(126)
        .decorated(false)
        .resizable(false)
        .build();
    window.add_css_class("pemdash");

    let input = Entry::builder()
        .placeholder_text("Type an expression, for example: 50 / sqrt(3) * 1e5")
        .hexpand(true)
        .build();
    input.add_css_class("expression");

    let result = Label::builder()
        .label("Enter an expression")
        .halign(Align::Start)
        .hexpand(true)
        .selectable(true)
        .build();
    result.add_css_class("result");

    let copy = Button::with_label("Copy");
    copy.set_sensitive(false);
    copy.add_css_class("copy");

    let result_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
    result_row.append(&result);
    result_row.append(&copy);

    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    content.append(&input);
    content.append(&result_row);
    window.set_child(Some(&content));

    let current_result = Rc::new(RefCell::new(None::<String>));

    {
        let result = result.clone();
        let copy = copy.clone();
        let current_result = current_result.clone();
        input.connect_changed(move |input| {
            result.remove_css_class("error");
            copy.set_label("Copy");

            let expression = input.text();
            if expression.trim().is_empty() {
                result.set_text("Enter an expression");
                copy.set_sensitive(false);
                *current_result.borrow_mut() = None;
                return;
            }

            match pemdash::calculate(&expression) {
                Ok(value) => {
                    let text = format_result(value);
                    result.set_text(&format!("= {text}"));
                    copy.set_sensitive(true);
                    *current_result.borrow_mut() = Some(text);
                }
                Err(error) => {
                    result.set_text(&error);
                    result.add_css_class("error");
                    copy.set_sensitive(false);
                    *current_result.borrow_mut() = None;
                }
            }
        });
    }

    {
        let current_result = current_result.clone();
        copy.connect_clicked(move |button| {
            if let Some(text) = current_result.borrow().as_deref()
                && let Some(display) = gdk::Display::default()
            {
                display.clipboard().set_text(text);
                button.set_label("Copied");
            }
        });
    }

    {
        let current_result = current_result.clone();
        let window = window.clone();
        input.connect_activate(move |_| {
            if let Some(text) = current_result.borrow().as_deref()
                && let Some(display) = gdk::Display::default()
            {
                display.clipboard().set_text(text);
                window.close();
            }
        });
    }

    let keys = EventControllerKey::new();
    {
        let window = window.clone();
        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                window.close();
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

fn format_result(value: f64) -> String {
    if value == 0.0 {
        "0".into()
    } else {
        value.to_string()
    }
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(
        r#"
        window.pemdash {
            background: #20242c;
        }

        entry.expression {
            min-height: 38px;
            padding: 0 12px;
            font-family: monospace;
            font-size: 17px;
        }

        label.result {
            padding-left: 12px;
            color: #d8dee9;
            font-family: monospace;
            font-size: 15px;
        }

        label.result.error {
            color: #e06c75;
        }

        button.copy {
            min-width: 72px;
        }
        "#,
    );

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
    use super::format_result;

    #[test]
    fn formats_whole_numbers_without_decimal_noise() {
        assert_eq!(format_result(6.0), "6");
        assert_eq!(format_result(-0.0), "0");
    }
}
