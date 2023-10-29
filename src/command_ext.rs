use std::process::Command;

use miette::miette;
use miette::Context;
use miette::IntoDiagnostic;

pub trait CommandExt {
    /// Run the command and error if it failed.
    fn status_checked(&mut self) -> miette::Result<()>;

    /// Run the command and produce stdout, decoded as UTF-8.
    fn stdout_checked_utf8(&mut self) -> miette::Result<String>;

    /// Display the command as a string, suitable for user output.
    ///
    /// Arguments and program names should be quoted with [`shell_words::quote`].
    fn display(&self) -> String;

    /// Display the program name as a string, suitable for user output.
    fn display_program(&self) -> String;
}

impl CommandExt for Command {
    fn status_checked(&mut self) -> miette::Result<()> {
        let cmd = self.display();
        tracing::debug!("$ {cmd}");
        let status = self
            .status()
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to execute: {cmd}"))?;

        let program = self.display_program();
        if status.success() {
            tracing::debug!("{program} completed successfully");
            Ok(())
        } else {
            Err(miette!(
                "{program} failed with exit code {status}: {}",
                self.display()
            ))
        }
    }

    fn stdout_checked_utf8(&mut self) -> miette::Result<String> {
        let cmd = self.display();
        tracing::debug!("$ {cmd}");
        let output = self
            .output()
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to execute: {cmd}"))?;

        let program = self.display_program();
        let status = output.status;
        if status.success() {
            tracing::debug!("{program} completed successfully");
            let stdout = String::from_utf8(output.stdout.clone())
                .into_diagnostic()
                .wrap_err_with(|| {
                    format!(
                        "Failed to decode stdout as UTF-8: {}",
                        String::from_utf8_lossy(&output.stdout)
                    )
                })?;
            let stdout = stdout.trim().to_owned();
            Ok(stdout)
        } else {
            let mut message = format!(
                "{program} failed with exit code {status}: {}",
                self.display()
            );

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stdout = stdout.trim();
            if !stdout.is_empty() {
                message.push_str(&format!("\nStdout: {stdout}"));
            }

            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr = stderr.trim();
            if !stderr.is_empty() {
                message.push_str(&format!("\nStderr: {stderr}"));
            }

            Err(miette!("{message}"))
        }
    }

    fn display(&self) -> String {
        let program = self.get_program().to_string_lossy();

        let args = self.get_args().map(|arg| arg.to_string_lossy());

        let tokens = std::iter::once(program).chain(args);

        shell_words::join(tokens)
    }

    fn display_program(&self) -> String {
        let program = self.get_program().to_string_lossy();
        shell_words::quote(&program).into_owned()
    }
}
