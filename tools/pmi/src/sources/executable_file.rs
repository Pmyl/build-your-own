use std::{
    env, fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use build_your_own_utils::my_own_error::{DescribableError, MyOwnResult};

use crate::{
    applications::{ApplicationName, applications_folder},
    sources::SourceManager,
};

pub(crate) struct ExecutableFile;

impl SourceManager for ExecutableFile {
    fn install<'a, Args: IntoIterator<Item = &'a str>>(
        &self,
        application: &'a ApplicationName,
        args: Args,
    ) -> MyOwnResult<()> {
        let args = args.into_iter().collect::<Vec<_>>();
        let file_path_string = args
            .first()
            .ok_or_else(|| "Executable file installation should have one argument containing the path to the file")?
            .to_string();
        let file_path = Path::new(&file_path_string);
        if !file_path.is_file() {
            return Err(format!(
                "File path for executable file installation is not a file: [{}]",
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

        let symlink_target = format!("/usr/bin/{}", application.0);
        println!(
            "# Creating a symlink between {} and {}",
            cached_file_path.to_string_lossy(),
            symlink_target
        );
        symlink(cached_file_path, symlink_target)
            .with_error_description(|| "When symlinking executable file")?;

        Ok(())
    }

    fn uninstall<'a>(&self, _: &'a ApplicationName) -> MyOwnResult<()> {
        todo!("Just go and delete it manually from both /usr/bin and pmi folder")
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
