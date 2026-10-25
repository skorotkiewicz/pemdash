use gtk4::{gio, glib};
use std::ffi::OsStr;
use std::time::Duration;

/// A session-specific helper that owns clipboard data after pemdash exits.
pub(crate) struct ClipboardHelper {
    command: &'static [&'static str],
    package: &'static str,
}

impl ClipboardHelper {
    /// Selects the helper for the actual GDK display, not the session environment.
    pub(crate) fn for_backend(backend: &str) -> Result<Self, String> {
        match backend {
            "GdkWaylandDisplay" => Ok(Self {
                command: &["wl-copy", "--type", "text/plain;charset=utf-8"],
                package: "wl-clipboard",
            }),
            "GdkX11Display" => Ok(Self {
                command: &["xclip", "-selection", "clipboard", "-in", "-silent"],
                package: "xclip",
            }),
            _ => Err("Persistent clipboard copying requires a Wayland or X11 display".into()),
        }
    }

    /// Sends text over stdin and waits for a successful, persistent handoff.
    pub(crate) async fn copy_text(&self, text: &str) -> Result<(), String> {
        run_helper(self.command, self.package, text, Duration::from_secs(5)).await
    }
}

async fn run_helper(
    command: &[&str],
    package: &str,
    text: &str,
    timeout: Duration,
) -> Result<(), String> {
    let Some(program) = command.first() else {
        return Err("No clipboard helper command was provided".into());
    };
    let arguments: Vec<_> = command.iter().map(OsStr::new).collect();
    // Both helpers fork a persistent owner. Do not wait on output pipes that
    // the background owner could keep open after the initial process exits.
    let process = gio::Subprocess::newv(
        &arguments,
        gio::SubprocessFlags::STDIN_PIPE
            | gio::SubprocessFlags::STDOUT_SILENCE
            | gio::SubprocessFlags::STDERR_SILENCE,
    )
    .map_err(|error| format!("Could not start {program}. Install {package}. {error}"))?;

    match glib::future_with_timeout(
        timeout,
        process.communicate_utf8_future(Some(text.to_owned())),
    )
    .await
    {
        Ok(Ok(_)) if process.is_successful() => Ok(()),
        Ok(Ok(_)) => Err(format!(
            "{program} could not copy the result. Check your clipboard session and try again."
        )),
        Ok(Err(error)) => {
            process.force_exit();
            let _ = process.wait_future().await;
            Err(format!("Could not send the result to {program}: {error}"))
        }
        Err(_) => {
            process.force_exit();
            let _ = process.wait_future().await;
            Err(format!("{program} did not finish copying. Try again."))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(command: &[&str], text: &str, timeout: Duration) -> Result<(), String> {
        let context = glib::MainContext::new();
        context
            .with_thread_default(|| {
                context.block_on(run_helper(command, "test-package", text, timeout))
            })
            .expect("Each test owns its GLib main context")
    }

    #[test]
    fn selects_helper_for_actual_display_backend() {
        let helper = ClipboardHelper::for_backend("GdkWaylandDisplay")
            .expect("Wayland should have a clipboard helper");
        assert_eq!(helper.command[0], "wl-copy");
        assert_eq!(helper.package, "wl-clipboard");
        let helper = ClipboardHelper::for_backend("GdkX11Display")
            .expect("X11 should have a clipboard helper");
        assert_eq!(helper.command[0], "xclip");
        assert_eq!(helper.package, "xclip");
        assert!(
            ClipboardHelper::for_backend("GdkBroadwayDisplay").is_err(),
            "Unsupported backends must not report a persistent copy"
        );
    }

    #[test]
    fn sends_result_over_stdin_and_waits_for_success() {
        run(
            &[
                "/bin/sh",
                "-c",
                "IFS= read -r value; test \"$value\" = '-408'",
            ],
            "-408",
            Duration::from_secs(2),
        )
        .expect("The helper should receive the exact result over stdin");
    }

    #[test]
    fn rejects_empty_command() {
        let error = run(&[], "408", Duration::from_secs(2))
            .expect_err("An empty command cannot copy anything");
        assert!(error.contains("No clipboard helper"), "{error}");
    }

    #[test]
    fn reports_missing_helper_and_required_package() {
        let error = run(
            &["/pemdash-nonexistent-clipboard-helper"],
            "408",
            Duration::from_secs(2),
        )
        .expect_err("A missing helper must fail");
        assert!(error.contains("Install test-package"), "{error}");
    }

    #[test]
    fn reports_helper_failure() {
        run(
            &["/bin/sh", "-c", "IFS= read -r value; exit 7"],
            "408",
            Duration::from_secs(2),
        )
        .expect_err("A nonzero helper exit must fail the copy operation");
    }

    #[test]
    fn stops_helper_that_does_not_finish() {
        let error = run(
            &["/bin/sh", "-c", "exec sleep 5"],
            "408",
            Duration::from_millis(50),
        )
        .expect_err("A helper that does not exit must time out");
        assert!(error.contains("did not finish copying"), "{error}");
    }
}
