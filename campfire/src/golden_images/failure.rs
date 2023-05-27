use std::process::ExitStatus;

#[derive(Debug)]
pub(super) struct Failure {
    test: String,
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
    pub(super) fn log(&self) {
        log::error!("{} failed with status {}", self.test, self.status);
        log::error!("stdout:");
        self.stdout.lines().for_each(|line| eprintln!("{line}"));
        log::error!("stderr:");
        self.stderr.lines().for_each(|line| eprintln!("{line}"));
        eprintln!(); // Space between consecutive errors.
    }
}
