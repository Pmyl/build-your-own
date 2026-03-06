use std::{
    env, fs,
    io::{stderr, stdout},
    path::{Path, PathBuf},
    process::Command,
};

use build_your_own_utils::my_own_error::{DescribableError, MyOwnError, MyOwnResult};

use crate::{
    applications::{ApplicationName, applications_folder},
    sources::SourceManager,
};

pub(crate) struct InstallSh;

impl SourceManager for InstallSh {
    fn install<'a, Args: IntoIterator<Item = &'a str>>(
        &self,
        _: &'a ApplicationName,
        args: Args,
    ) -> MyOwnResult<()> {
        let args = args.into_iter().collect::<Vec<_>>();
        let file_path = if args.first().map_or(false, |arg| *arg == "--curl") {
            let url = args
                .get(1)
                .ok_or_else(|| "curl command requires a URL argument")?;

            let temp_file = std::env::temp_dir().join("downloaded_script.sh");

            let status = Command::new("curl")
                .args(&[
                    "-fsSL",
                    url,
                    "-o",
                    &temp_file.to_string_lossy().into_owned(),
                ])
                .status()?;

            if !status.success() {
                return Err(MyOwnError::ActualError(
                    format!("Failed to download file from {}", url).into(),
                ));
            }

            to_absolute(&temp_file)
        } else {
            let file_path_string = args
                .first()
                .ok_or_else(|| "Install sh installation should have one argument containing the path to the file")?
                .to_string();
            let file_path = Path::new(&file_path_string);
            if !file_path.is_file() {
                return Err(format!(
                    "File path for instll sh installation is not a file: [{}]",
                    file_path.to_string_lossy()
                )
                .into());
            }

            let file_path = to_absolute(file_path);
            let cached_file_path = to_absolute(
                &Path::new(&applications_folder()).join(file_path.file_name().unwrap()),
            );

            if file_path == cached_file_path {
                println!(
                    "# File already in applications folder {}",
                    applications_folder()
                );
            } else {
                println!(
                    "# Moving file {} to {}",
                    file_path.to_string_lossy(),
                    cached_file_path.to_string_lossy()
                );
                if fs::rename(&file_path, &cached_file_path).is_err() {
                    {
                        fs::copy(&file_path, &cached_file_path)?;
                        fs::remove_file(&file_path)
                    }
                    .with_error_description(
                        || "When moving executable file in applications folder",
                    )?;
                }
            }

            cached_file_path
        };

        let status = Command::new("sh")
            .args(&vec![file_path])
            .stdout(stdout())
            .stderr(stderr())
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(MyOwnError::ActualError("# Couldn't install".into()))
        }
    }

    fn uninstall<'a>(&self, _: &'a ApplicationName) -> MyOwnResult<()> {
        todo!("Read the sh script")
    }

    fn has_application(&self, _: &str) -> MyOwnResult<bool> {
        Ok(false)
    }
}

fn to_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().unwrap().join(path)
    }
}
