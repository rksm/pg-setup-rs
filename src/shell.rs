use std::{
    io::{BufRead, Read},
    process,
};

use crate::error::Error;

use super::error::Result;

pub fn cmd(cmd: impl AsRef<str>) -> Result<process::Child> {
    trace!("running command {}", cmd.as_ref());
    let mut args = cmd.as_ref().split(' ');
    let cmd = args.next().unwrap();
    cmd_with_args(cmd, args)
}

pub fn cmd_with_args<I, S>(cmd: impl AsRef<str>, args: I) -> Result<process::Child>
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<std::ffi::OsStr> + std::fmt::Debug,
{
    trace!(
        "running command {} {:?}",
        cmd.as_ref(),
        args.clone().into_iter().collect::<Vec<_>>()
    );

    let mut proc = std::process::Command::new(cmd.as_ref())
        .args(args)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .map_err(Error::ProcessError)?;

    fn fwd_stream(stream: Option<impl Read + Send + 'static>) {
        if let Some(stream) = stream {
            std::thread::spawn(move || {
                let mut reader = std::io::BufReader::new(stream);
                let mut line = String::new();
                loop {
                    match reader.read_line(&mut line) {
                        Err(err) => {
                            eprintln!("error reading line: {err}");
                        }
                        Ok(0) => {
                            break;
                        }
                        Ok(_) => {
                            print!("{line}");
                            line.clear();
                        }
                    }
                }
            });
        }
    }

    fwd_stream(proc.stdout.take());
    fwd_stream(proc.stderr.take());

    Ok(proc)
}
