use std::{
    env, fs,
    io::{stderr, stdout},
    path::{Path, PathBuf},
    process::Command,
};

use build_your_own_utils::my_own_error::{DescribableError, MyOwnError, MyOwnResult};

use crate::{ApplicationInstructions, applications_folder, sources::SourceManager};

pub(crate) struct InstallSh;

impl SourceManager for InstallSh {
    fn install(&self, application: &ApplicationInstructions) -> MyOwnResult<()> {
        let file_path_string = application
            .args
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
        let cached_file_path =
            to_absolute(&Path::new(&applications_folder()).join(file_path.file_name().unwrap()));

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
                .with_error_description(|| "When moving executable file in applications folder")?;
            }
        }

        let status = Command::new("sh")
            .args(&vec![cached_file_path])
            .stdout(stdout())
            .stderr(stderr())
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(MyOwnError::ActualError("# Couldn't install".into()))
        }
    }

    fn uninstall(&self, _: &ApplicationInstructions) -> MyOwnResult<()> {
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
