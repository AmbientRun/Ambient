use std::{fmt::Display, process::ExitStatus};

#[derive(Debug)]
pub(super) struct Failure {
    pub test: String,
    stdout: String,
    stderr: String,
    status: ExitStatus,
}

impl Failure {
    pub(super) fn from_output(test: String, output: &std::process::Output) -> Self {
        let stdout =
            String::from_utf8(output.stdout.clone()).expect("stdout must be a valid UTF-8");
        let stderr =
            String::from_utf8(output.stderr.clone()).expect("stderr must be a valid UTF-8");
        Failure {
            test,
            stdout,
            stderr,
            status: output.status,
        }
    }
}

impl Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} failed with status {}", self.test, self.status)?;
        writeln!(f, "stdout: ")?;

        self.stdout
            .lines()
            .try_for_each(|line| writeln!(f, "    {}", line))?;
        writeln!(f, "stderr: ")?;

        self.stderr
            .lines()
            .try_for_each(|line| writeln!(f, "    {}", line))?;

        Ok(())
    }
}
