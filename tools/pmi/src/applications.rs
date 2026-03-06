use std::{
    borrow::Cow,
    fmt::Display,
    fs::OpenOptions,
    io::{Read, Write, stdin},
};

use build_your_own_utils::{
    fuzzy_search::fuzzy_search,
    my_own_error::{DescribableError, MyOwnError, MyOwnResult},
};

use crate::sources::{Source, SourceInstructions};

#[derive(PartialEq, Clone)]
pub(crate) struct ApplicationName<'a>(pub Cow<'a, str>);

pub(crate) struct RequestApplication<'a> {
    pub name: ApplicationName<'a>,
    pub source_instruction: Option<SourceInstructions<'a>>,
}

impl<'a> RequestApplication<'a> {
    pub fn as_application(&self) -> Option<Application<'a>> {
        self.source_instruction
            .as_ref()
            .map(|source_instruction| Application {
                name: self.name.clone(),
                source_instruction: source_instruction.clone(),
            })
    }
}

pub(crate) struct Application<'a> {
    pub name: ApplicationName<'a>,
    pub source_instruction: SourceInstructions<'a>,
}

impl<'a> Display for Application<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (through) {}",
            self.name.0, self.source_instruction.source
        )?;
        if !self.source_instruction.args.is_empty() {
            write!(f, "{}", self.source_instruction.args.join(" "))?;
        }
        Ok(())
    }
}

pub(crate) struct PersistedApplication<'a> {
    pub name: ApplicationName<'a>,
    pub source_instructions: Vec<SourceInstructions<'a>>,
}

impl<'a> PersistedApplication<'a> {
    pub fn from_application(app: Application<'a>) -> Self {
        Self {
            name: app.name,
            source_instructions: vec![app.source_instruction],
        }
    }

    pub fn identify_same_application(&self, application: &Application<'a>) -> bool {
        self.name == application.name
            && self
                .source_instructions
                .contains(&application.source_instruction)
    }

    pub fn serialize(&self) -> String {
        fn serialize_str(out: &mut String, s: &str) {
            out.push_str(&s.len().to_string());
            out.push(':');
            out.push_str(s);
        }

        let mut serialized = String::new();

        serialize_str(&mut serialized, &self.name.0);

        serialized.push_str(&self.source_instructions.len().to_string());
        serialized.push(':');

        for instr in &self.source_instructions {
            serialize_str(&mut serialized, &instr.source.to_string());

            serialized.push_str(&instr.args.len().to_string());
            serialized.push(':');

            for arg in &instr.args {
                serialize_str(&mut serialized, arg);
            }
        }

        serialized
    }

    pub fn deserialize(mut raw: &str) -> MyOwnResult<Self> {
        fn deserialize_str(input: &mut &str) -> MyOwnResult<String> {
            let (len_str, rest) = input
                .split_once(':')
                .ok_or_else(|| "Stored application is not in the correct format")?;
            let len: usize = len_str.parse()?;

            let (value, remaining) = rest.split_at(len);
            *input = remaining;

            Ok(value.to_string())
        }

        let application_name = ApplicationName(Cow::Owned(deserialize_str(&mut raw)?));

        let (sources_count_raw, rest) = raw
            .split_once(':')
            .ok_or_else(|| "Stored application is not in the correct format")?;
        let sources_count: usize = sources_count_raw.parse()?;
        raw = rest;

        let mut sources_instructions = Vec::with_capacity(sources_count);

        for _ in 0..sources_count {
            let source: Source = Source::from_persisted(deserialize_str(&mut raw)?);
            let (args_count_raw, rest) = raw
                .split_once(':')
                .ok_or_else(|| "Stored application is not in the correct format")?;
            let args_count: usize = args_count_raw.parse()?;
            raw = rest;

            let mut args = Vec::with_capacity(args_count);

            for _ in 0..args_count {
                let arg = deserialize_str(&mut raw)?;
                args.push(arg);
            }

            sources_instructions.push(SourceInstructions {
                source: source,
                args: args.into_iter().map(|arg| Cow::Owned(arg)).collect(),
            })
        }

        Ok(Self {
            name: application_name,
            source_instructions: sources_instructions,
        })
    }
}

impl<'a> Display for PersistedApplication<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((first, others)) = self.source_instructions.split_first() {
            write!(f, "{} (through) {}", self.name.0, first.source)?;
            if !first.args.is_empty() {
                write!(f, " {}", first.args.join(" "))?;
            }

            for other in others {
                writeln!(f, "   ...and (through) {}", first.source)?;
                if !other.args.is_empty() {
                    write!(f, " {}", other.args.join(" "))?;
                }
            }
        }

        Ok(())
    }
}

pub(crate) struct Persistence<'a> {
    pub list: Vec<PersistedApplication<'a>>,
    readonly: bool,
}

impl<'a> Persistence<'a> {
    pub fn from_file(persisted: bool) -> MyOwnResult<Self> {
        let readonly = !persisted;

        std::fs::create_dir_all(applications_folder())
            .with_error_description(|| format!("Error while creating {}", applications_folder()))?;
        let file_reader = OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(applications_file())
            .with_error_description(|| format!("Error while loading {}", applications_file()))?;

        let list = Persistence::<'a>::list_from_reader(file_reader)?;
        let list_source = applications_file();

        println!("# Applications list from [{}]", list_source);
        Ok(Persistence { list, readonly })
    }

    pub fn from_stdin(readonly: bool) -> MyOwnResult<Self> {
        let list = Persistence::<'a>::list_from_reader(stdin())?;
        let list_source = "stdin".to_string();

        println!("# Applications list from [{}]", list_source);
        Ok(Persistence { list, readonly })
    }

    pub fn list_from_reader(mut reader: impl Read) -> MyOwnResult<Vec<PersistedApplication<'a>>> {
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .with_error_description(|| format!("Error while reading applications"))?;

        content
            .lines()
            .map(|l| PersistedApplication::deserialize(l))
            .collect()
    }

    pub fn add_many(&mut self, applications: Vec<Application<'a>>) -> MyOwnResult<()> {
        for application in applications {
            self.add_to_list(application);
        }
        self.persist()
    }

    pub fn add(&mut self, application: Application<'a>) -> MyOwnResult<()> {
        self.add_to_list(application);
        self.persist()
    }

    fn add_to_list(&mut self, application: Application<'a>) {
        let index = self.list.iter().position(|pa| pa.name == application.name);
        if let Some(index) = index {
            let persisted_app = self.list.get_mut(index).unwrap();
            persisted_app
                .source_instructions
                .push(application.source_instruction);
        } else {
            self.list
                .push(PersistedApplication::from_application(application));
        }
    }

    pub fn remove_specific(&mut self, application: &Application<'a>) -> MyOwnResult<()> {
        let app_index = self
            .list
            .iter()
            .position(|pa| pa.name == application.name)
            .ok_or_else(|| {
                MyOwnError::ActualError("Application not in list, nothing to uninstall".into())
            })?;

        let source_instruction_index = self.list[app_index]
            .source_instructions
            .iter()
            .position(|si| si.source == application.source_instruction.source)
            .ok_or_else(|| {
                MyOwnError::ActualError(
                    "Application not installed with requested source, nothing to uninstall".into(),
                )
            })?;

        if self.list[app_index].source_instructions.len() == 1 {
            self.list.remove(app_index);
        } else {
            self.list[app_index]
                .source_instructions
                .remove(source_instruction_index);
        }

        self.persist()
    }

    pub fn remove(&mut self, application_name: &ApplicationName<'a>) -> MyOwnResult<()> {
        let app_index = self
            .list
            .iter()
            .position(|pa| &pa.name == application_name)
            .ok_or_else(|| {
                MyOwnError::ActualError("Application not in list, nothing to uninstall".into())
            })?;

        self.list.remove(app_index);
        self.persist()
    }

    pub fn remove_all(&mut self) -> MyOwnResult<()> {
        self.list.clear();
        self.persist()
    }

    pub fn get_application_by_name<'b>(
        &self,
        application: &'b str,
    ) -> Option<&PersistedApplication<'a>> {
        self.list.iter().find(|app| &app.name.0 == application)
    }

    fn persist(&self) -> MyOwnResult<()> {
        if !self.readonly {
            let mut file = OpenOptions::new()
                .write(true)
                .append(false)
                .truncate(true)
                .open(applications_file())
                .with_error_description(|| {
                    format!("Error while loading {}", applications_file())
                })?;

            for pa in &self.list {
                writeln!(file, "{}", pa.serialize())?;
            }
            file.flush()?;
        }

        Ok(())
    }

    pub fn is_already_installed(
        &'a self,
        request: &RequestApplication<'a>,
    ) -> AlreadyInstalled<'a> {
        let indices = fuzzy_search(
            &request.name.0,
            &self
                .list
                .iter()
                .map(|a| a.name.0.as_ref())
                .collect::<Vec<_>>(),
            3,
        );

        let similar_matches = indices
            .into_iter()
            .map(|i| self.list.get(i).unwrap())
            .collect::<Vec<_>>();

        let perfect_match = request.as_application().and_then(|application| {
            self.list
                .iter()
                .find(|pa| pa.identify_same_application(&application))
        });

        AlreadyInstalled {
            perfect_match,
            similar_matches,
        }
    }
}

pub(crate) struct AlreadyInstalled<'a> {
    pub perfect_match: Option<&'a PersistedApplication<'a>>,
    pub similar_matches: Vec<&'a PersistedApplication<'a>>,
}

pub(crate) fn applications_folder() -> String {
    std::env::var("PMI_DIR").unwrap_or_else(|_| {
        format!(
            "/home/{}/.pmi",
            std::env::var("SUDO_USER")
                .or_else(|_| std::env::var("USER"))
                .unwrap()
        )
    })
}

pub(crate) fn applications_file() -> String {
    format!("{}/applications", applications_folder())
}
