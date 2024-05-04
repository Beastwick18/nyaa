use std::{
    error::Error,
    io::{BufReader, Read as _},
    process::{Command, Stdio},
};

pub struct CommandBuilder {
    cmd: String,
}

impl CommandBuilder {
    pub fn new(cmd: String) -> Self {
        CommandBuilder { cmd }
    }

    pub fn sub(&mut self, pattern: &str, sub: &str) -> &mut Self {
        self.cmd = self.cmd.replace(pattern, sub);
        self
    }

    pub fn run<S: Into<Option<String>>>(&self, shell: S) -> Result<(), Box<dyn Error>> {
        let shell = Into::<Option<String>>::into(shell).unwrap_or(Self::default_shell());
        let cmds = shell.split_whitespace().collect::<Vec<&str>>();
        if let [base_cmd, args @ ..] = cmds.as_slice() {
            let cmd = Command::new(base_cmd)
                .args(args)
                .arg(&self.cmd)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn();

            let child = match cmd {
                Ok(child) => child,
                Err(e) => return Err(format!("{}:\nFailed to run:\n{}", self.cmd, e).into()),
            };
            let output = match child.wait_with_output() {
                Ok(output) => output,
                Err(e) => return Err(format!("{}:\nFailed to get output:\n{}", self.cmd, e).into()),
            };

            if output.status.code() != Some(0) {
                let mut err = BufReader::new(&*output.stderr);
                let mut err_str = String::new();
                err.read_to_string(&mut err_str).unwrap_or(0);
                return Err(format!(
                    "{}:\nExited with status code {}:\n{}",
                    self.cmd, output.status, err_str
                )
                .into());
            }
            Ok(())
        } else {
            Err(format!("Shell command is not properly formatted:\n{}", shell).into())
        }
    }

    pub fn default_shell() -> String {
        #[cfg(windows)]
        return "powershell.exe -Command".to_owned();
        #[cfg(unix)]
        return "sh -c".to_owned();
    }
}
