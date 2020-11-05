use crate::options::Options;
use snafu::{Backtrace, ResultExt, Snafu};
use std::path::PathBuf;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not write to log file {} error: {}", filename.display(), source))]
    WriteLog {
        filename: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Log file {} initialization error: {}", filename.display(), source))]
    InitializeWriteLog {
        filename: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Log file {} initialization error: {}", filename.display(), source))]
    InitializeLog {
        filename: PathBuf,
        source: log::SetLoggerError,
        backtrace: Backtrace,
    },
}

pub(crate) fn setup_logger(options: &Options) -> Result<(), Error> {
    let log_file = PathBuf::from(&options.log_file);
    {
        // try writing in the log so we
        // can output a meaningful message
        // in case we don't have enough
        // privileges
        let _file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&log_file)
            .context(WriteLog {
                filename: &log_file,
            })?;
    }

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(options.log_level)
        .chain(std::io::stdout())
        .chain(
            fern::log_file(log_file.clone()).context(InitializeWriteLog {
                filename: &log_file,
            })?,
        )
        .apply()
        .context(InitializeLog {
            filename: &log_file,
        })?;
    Ok(())
}
