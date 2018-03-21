//! Data structures and behavior for generated snapshot tests.

use std::env;
use std::fmt::Debug;
use std::path::PathBuf;

use diff::{self, Result as Diff};

use report::Report;
use store::Store;

lazy_static! {
    static ref STORE: Store = Store::load().expect("failed to load snaptest store");
}

#[derive(Copy, Clone, Debug)]
pub struct Test {
    file: &'static str,
    name: &'static str,
    path: &'static str,
    uuid: &'static str,
    ret: &'static str,
}

#[derive(Copy, Clone, Debug)]
pub struct Builder {
    file: Option<&'static str>,
    name: Option<&'static str>,
    path: Option<&'static str>,
    uuid: Option<&'static str>,
    ret: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Failure(String, String),
    Success,
}

impl Test {
    pub fn builder() -> Builder {
        Builder {
            file: None,
            name: None,
            path: None,
            uuid: None,
            ret: None,
        }
    }

    pub fn file(&self) -> &'static str {
        self.file
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn path(&self) -> &'static str {
        self.path
    }

    pub fn uuid(&self) -> &'static str {
        self.uuid
    }

    pub fn ret(&self) -> &'static str {
        self.ret
    }

    pub(crate) fn basename(&self) -> Option<PathBuf> {
        let path = PathBuf::from(self.file);
        Some(PathBuf::from(path.file_name()?))
    }

    pub(crate) fn dirname(&self) -> ::Result<PathBuf> {
        let path = PathBuf::from(self.file);
        let cwd = env::current_dir()?;

        for component in cwd.components() {
            let part = component.as_os_str();

            if path.starts_with(part) {
                path.strip_prefix(part)?;
            }
        }

        match path.parent() {
            Some(parent) => Ok(parent.to_owned()),
            None => Ok(PathBuf::new()),
        }
    }
}

impl Builder {
    pub fn run<T: Debug>(&mut self, f: fn() -> ::Result<T>) -> ::Result<Report> {
        macro_rules! required {
            ( $self:ident.$field:ident ) => (match $self.$field.take() {
                Some(inner) => inner,
                None => bail!("{}  is a required field", stringify!($field)),
            })
        }

        let test = Test {
            file: required!(self.file),
            name: required!(self.name),
            path: required!(self.path),
            uuid: required!(self.uuid),
            ret: required!(self.ret),
        };

        STORE.compare(
            test.uuid(),
            || {
                let key = test.uuid().to_owned();
                let value = format!("{:#?}", f()?);

                STORE.insert(key, value)?;
                STORE.save()?;

                Ok(Report::new(test, Outcome::Success))
            },
            |left| {
                let right = format!("{:#?}", f()?);
                let outcome = if left == right {
                    Outcome::Success
                } else {
                    Outcome::Failure(left.to_owned(), right)
                };

                Ok(Report::new(test, outcome))
            },
        )
    }

    pub fn file(&mut self, value: &'static str) -> &mut Builder {
        self.file = Some(value);
        self
    }

    pub fn name(&mut self, value: &'static str) -> &mut Builder {
        self.name = Some(value);
        self
    }

    pub fn path(&mut self, value: &'static str) -> &mut Builder {
        self.path = Some(value);
        self
    }

    pub fn uuid(&mut self, value: &'static str) -> &mut Builder {
        self.uuid = Some(value);
        self
    }

    pub fn ret(&mut self, value: &'static str) -> &mut Builder {
        self.ret = Some(value);
        self
    }
}

impl Outcome {
    pub fn diff(&self) -> Vec<Diff<&str>> {
        match *self {
            Outcome::Failure(ref l, ref r) => diff::lines(l, r),
            Outcome::Success => Vec::new(),
        }
    }

    pub fn was_successful(&self) -> bool {
        match *self {
            Outcome::Failure(_, _) => false,
            Outcome::Success => true,
        }
    }
}
