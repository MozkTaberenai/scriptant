use super::*;

use std::marker::PhantomData;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::process::{ChildStdin, ChildStdout};

use once_cell::sync::Lazy;
static ECHO_PREFIX: Lazy<String> = Lazy::new(|| echo::prefix("cmd"));

#[derive(Debug)]
pub enum Error {
    Exit { code: i32 },
    Terminated,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Exit { code } => write!(f, "Exit with error status code: {code}"),
            Error::Terminated => write!(f, "Terminated by signal"),
        }
    }
}

impl std::error::Error for Error {}

fn echo_command_info(command: &Command, echo: &mut Echo) {
    if let Some(current_dir) = command.get_current_dir() {
        let current_dir = format!(
            "{}{}",
            "cwd:".bright_black(),
            current_dir.to_string_lossy().underline().bright_black(),
        );
        echo.put(current_dir);
    }

    let envs = command.get_envs();
    if envs.len() > 0 {
        for (k, v) in envs {
            if let Some(v) = v {
                echo.put(format!(
                    "{}{}{}{}",
                    "env:".bright_black(),
                    k.to_string_lossy().underline().bright_black(),
                    "=".bright_black(),
                    v.to_string_lossy().underline().bright_black(),
                ));
            } else {
                echo.put(format!(
                    "{}{}{}",
                    "env:".bright_black(),
                    "!".bright_black(),
                    k.to_string_lossy().underline().bright_black(),
                ));
            }
        }
    }

    echo.put(
        command
            .get_program()
            .to_string_lossy()
            .bold()
            .cyan()
            .to_string(),
    );

    for arg in command.get_args() {
        echo.put(arg.to_string_lossy().underline().bold().to_string());
    }
}

pub enum UnknownStdio {}

pub struct Cmd<I = UnknownStdio, O = UnknownStdio> {
    inner: Command,
    quiet: bool,
    _marker: PhantomData<fn() -> (I, O)>,
}

impl<I, O> Cmd<I, O> {
    pub fn get_inner_ref(&self) -> &Command {
        &self.inner
    }

    pub fn get_inner_mut(&mut self) -> &mut Command {
        &mut self.inner
    }

    pub fn pipein(self) -> Cmd<ChildStdin, O> {
        Cmd {
            inner: self.inner,
            quiet: self.quiet,
            _marker: PhantomData,
        }
    }

    pub fn pipeout(self) -> Cmd<I, ChildStdout> {
        Cmd {
            inner: self.inner,
            quiet: self.quiet,
            _marker: PhantomData,
        }
    }

    pub fn stdin(mut self, cfg: impl Into<Stdio>) -> Cmd<UnknownStdio, O> {
        self.inner.stdin(cfg);
        Cmd {
            inner: self.inner,
            quiet: self.quiet,
            _marker: PhantomData,
        }
    }

    pub fn stdout(mut self, cfg: impl Into<Stdio>) -> Cmd<I, UnknownStdio> {
        self.inner.stdout(cfg);
        Cmd {
            inner: self.inner,
            quiet: self.quiet,
            _marker: PhantomData,
        }
    }

    pub fn current_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.inner.current_dir(dir);
        self
    }

    pub fn env(mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) -> Self {
        self.inner.env(key, val);
        self
    }

    pub fn env_clear(mut self) -> Self {
        self.inner.env_clear();
        self
    }

    pub fn env_remove(mut self, key: impl AsRef<OsStr>) -> Self {
        self.inner.env_remove(key);
        self
    }

    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.inner.arg(arg);
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Self {
        self.inner.args(args);
        self
    }

    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    fn _echo(&self, pipein: bool, pipeout: bool) -> Echo {
        let mut echo = Echo::new();
        if self.quiet {
            echo.quiet();
        }
        echo.put(&*ECHO_PREFIX);
        if pipein {
            echo.put("->|".magenta());
        }
        echo_command_info(&self.inner, &mut echo);
        if pipeout {
            echo.put("|->".magenta());
        }
        echo
    }

    fn _spawn(
        mut self,
        pipein: bool,
        pipeout: bool,
    ) -> Result<(Child, Option<ChildStdin>, Option<ChildStdout>)> {
        self._echo(pipein, pipeout).end();

        if pipein {
            self.inner.stdin(Stdio::piped());
        }
        if pipeout {
            self.inner.stdout(Stdio::piped());
        }

        let mut child = self.inner.spawn()?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take();

        Ok((child, stdin, stdout))
    }
}

impl Cmd {
    pub fn new(program: impl AsRef<OsStr>) -> Self {
        let inner = Command::new(program);
        let quiet = false;
        Cmd {
            inner,
            quiet,
            _marker: PhantomData,
        }
    }

    pub fn spawn(self) -> Result<Child> {
        let (child, _, _) = self._spawn(false, false)?;
        Ok(child)
    }

    pub fn run(self) -> Result<()> {
        let status = self.spawn()?.wait()?;

        if !status.success() {
            let err = match status.code() {
                Some(code) => Error::Exit { code },
                None => Error::Terminated,
            };
            Err(err)?;
        }

        Ok(())
    }

    pub fn output(mut self) -> Result<Output> {
        let mut echo = self._echo(false, true);
        match self.inner.output() {
            Err(err) => {
                echo.end();
                Err(err)?
            }
            Ok(output) => {
                if !output.stdout.is_empty() {
                    echo.put(format!("stdout: {} bytes", output.stdout.len()).magenta());
                }
                if !output.stderr.is_empty() {
                    echo.put(format!("stderr: {} bytes", output.stderr.len()).magenta());
                }
                echo.end();
                Ok(output)
            }
        }
    }

    pub fn pipe(self, command: impl Into<Command>) -> Pipeline {
        Pipeline::from(self).pipe(command)
    }
}

impl Cmd<ChildStdin, UnknownStdio> {
    pub fn spawn(self) -> Result<(ChildStdin, Child)> {
        let (child, stdin, _) = self._spawn(true, false)?;
        Ok((stdin.unwrap(), child))
    }
}

impl Cmd<UnknownStdio, ChildStdout> {
    pub fn spawn(self) -> Result<(ChildStdout, Child)> {
        let (child, _, stdout) = self._spawn(false, true)?;
        Ok((stdout.unwrap(), child))
    }
}
impl Cmd<ChildStdin, ChildStdout> {
    pub fn spawn(self) -> Result<(ChildStdin, ChildStdout, Child)> {
        let (child, stdin, stdout) = self._spawn(true, true)?;
        Ok((stdin.unwrap(), stdout.unwrap(), child))
    }
}

impl<I, O> From<Cmd<I, O>> for Command {
    fn from(cmd: Cmd<I, O>) -> Self {
        cmd.inner
    }
}

pub struct Pipeline<I = UnknownStdio, O = UnknownStdio> {
    commands: Vec<Command>,
    quiet: bool,
    _marker: PhantomData<fn() -> (I, O)>,
}

impl<I, O> From<Cmd<I, O>> for Pipeline<I, O> {
    fn from(cmd: Cmd<I, O>) -> Self {
        Self {
            commands: vec![cmd.inner],
            quiet: cmd.quiet,
            _marker: PhantomData,
        }
    }
}

impl<I, O> Pipeline<I, O> {
    pub fn pipe(mut self, command: impl Into<Command>) -> Self {
        self.commands.push(command.into());
        self
    }

    pub fn pipein(self) -> Pipeline<ChildStdin, O> {
        Pipeline {
            commands: self.commands,
            quiet: self.quiet,
            _marker: PhantomData,
        }
    }

    pub fn pipeout(self) -> Pipeline<I, ChildStdout> {
        Pipeline {
            commands: self.commands,
            quiet: self.quiet,
            _marker: PhantomData,
        }
    }

    fn _echo(&self, pipein: bool, pipeout: bool) {
        let mut echo = Echo::new();
        if self.quiet {
            echo.quiet();
        }
        echo.put(&*ECHO_PREFIX);

        let mut iter = self.commands.iter();
        let first = iter.next().unwrap();
        if pipein {
            echo.put("->|".magenta());
        }
        echo_command_info(first, &mut echo);
        for command in iter {
            echo.put("|".magenta());
            echo_command_info(command, &mut echo);
        }
        if pipeout {
            echo.put("|->".magenta());
        }

        echo.end();
    }

    fn _spawn(
        self,
        pipein: bool,
        pipeout: bool,
    ) -> Result<(PipelineChildren, Option<ChildStdin>, Option<ChildStdout>)> {
        self._echo(pipein, pipeout);
        let mut children = Vec::with_capacity(self.commands.len());

        let mut last_stdout = match pipein {
            true => Some(Stdio::piped()),
            false => None,
        };

        let max_i = self.commands.len() - 1;

        for (i, mut command) in self.commands.into_iter().enumerate() {
            if let Some(stdout) = last_stdout.take() {
                command.stdin(stdout);
            }
            if i < max_i || pipeout {
                command.stdout(Stdio::piped());
            }
            let mut child = command.spawn()?;
            if i < max_i {
                last_stdout = child.stdout.take().map(Stdio::from);
            }
            children.push(child);
        }

        let first = children.first_mut().unwrap();
        let stdin = first.stdin.take();
        let last = children.last_mut().unwrap();
        let stdout = last.stdout.take();

        Ok((PipelineChildren { children }, stdin, stdout))
    }
}

impl Pipeline {
    pub fn spawn(self) -> Result<PipelineChildren> {
        let (children, _, _) = self._spawn(false, false)?;
        Ok(children)
    }

    pub fn run(self) -> Result<()> {
        let status = self.spawn()?.wait()?;

        let mut ok = vec![];
        let mut err = vec![];
        for status in status {
            match status.success() {
                true => ok.push(status),
                false => err.push(status),
            }
        }

        if !err.is_empty() {
            let err = match err[0].code() {
                Some(code) => Error::Exit { code },
                None => Error::Terminated,
            };
            Err(err)?;
        }

        Ok(())
    }
}

impl Pipeline<ChildStdin, UnknownStdio> {
    pub fn spawn(self) -> Result<(ChildStdin, PipelineChildren)> {
        let (children, stdin, _) = self._spawn(true, false)?;
        Ok((stdin.unwrap(), children))
    }
}

impl Pipeline<UnknownStdio, ChildStdout> {
    pub fn spawn(self) -> Result<(ChildStdout, PipelineChildren)> {
        let (children, _, stdout) = self._spawn(false, true)?;
        Ok((stdout.unwrap(), children))
    }
}

impl Pipeline<ChildStdin, ChildStdout> {
    pub fn spawn(self) -> Result<(ChildStdin, ChildStdout, PipelineChildren)> {
        let (children, stdin, stdout) = self._spawn(true, true)?;
        Ok((stdin.unwrap(), stdout.unwrap(), children))
    }
}

pub struct PipelineChildren {
    children: Vec<Child>,
}

impl PipelineChildren {
    pub fn wait(&mut self) -> Result<Vec<ExitStatus>> {
        let mut status = Vec::with_capacity(self.children.len());
        for child in &mut self.children {
            status.push(child.wait()?);
        }
        Ok(status)
    }
}

#[macro_export]
macro_rules! cmd {
    ($program:expr) => {
        Cmd::new($program)
    };
    ($program:expr, $($arg:expr),* $(,)?) => {
        Cmd::new($program)$(.arg($arg))*
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cmd() {
        let status = cmd!("sh", "-c", "")
            .spawn()
            .expect("fail to spawn")
            .wait()
            .expect("fail to wait");
        assert!(status.success());
        assert_eq!(status.code(), Some(0));
    }

    #[test]
    fn pipeout() {
        let (mut stdout, mut child) = cmd!("echo", "-n", "abcde")
            .pipeout()
            .spawn()
            .expect("fail to spawn");

        let mut out = String::new();
        stdout.read_to_string(&mut out).unwrap();
        assert_eq!(out, "abcde");

        let status = child.wait().expect("fail to wait");
        assert!(status.success());
        assert_eq!(status.code(), Some(0));
    }

    #[test]
    fn pipeinout() {
        let (mut stdin, mut stdout, mut child) = cmd!("tr", "[:lower:]", "[:upper:]")
            .pipein()
            .pipeout()
            .spawn()
            .expect("fail to spawn");

        std::thread::spawn(move || write!(stdin, "xyz"));
        let mut out = vec![];
        stdout.read_to_end(&mut out).unwrap();
        assert_eq!(&out, b"XYZ");

        let status = child.wait().expect("fail to wait");
        assert!(status.success());
        assert_eq!(status.code(), Some(0));
    }

    #[test]
    fn pipeline() {
        let (mut stdin, mut stdout, mut child) = cmd!("rev")
            .pipe(cmd!("tr", "[:lower:]", "[:upper:]"))
            .pipein()
            .pipeout()
            .spawn()
            .expect("fail to spawn");

        std::thread::spawn(move || write!(stdin, "xyz"));
        let mut out = String::new();
        stdout.read_to_string(&mut out).unwrap();
        assert_eq!(out.trim(), "ZYX");

        let status = child.wait().expect("fail to wait");
        assert!(status.iter().all(ExitStatus::success));
        assert!(status.iter().all(|s| s.code() == Some(0)));
    }
}
