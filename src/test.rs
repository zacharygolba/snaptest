use std::fmt::{self, Debug, Display, Formatter};
use std::env::current_dir;
use std::path::PathBuf;

use Error;
use diff::{Diff, Line};
use store::Store;

type Runner<T> = fn() -> Result<T, Error>;

lazy_static! {
    static ref STORE: Store = Store::load().expect("failed to load snaptest store");
}

#[derive(Clone, Debug)]
pub struct Test<T> {
    file: &'static str,
    ident: &'static str,
    module: &'static str,
    output: Option<String>,
    ret_ty: &'static str,
    runner: Runner<T>,
}

#[derive(Clone, Debug)]
pub struct Builder<T> {
    file: Option<&'static str>,
    ident: Option<&'static str>,
    module: Option<&'static str>,
    ret_ty: Option<&'static str>,
    runner: Option<Runner<T>>,
}

#[derive(Copy, Clone, Debug)]
struct Report<'a> {
    diff: &'a [Line<'a>],
    file: &'a str,
    ident: &'a str,
    module: &'a str,
    ret_ty: &'a str,
}

impl<T: Debug> Test<T> {
    pub fn builder() -> Builder<T> {
        Builder {
            file: None,
            ident: None,
            runner: None,
            module: None,
            ret_ty: None,
        }
    }

    pub fn run(self) -> Result<(), Error> {
        let key = String::new() + self.file + ":" + self.ident;
        let value = format!("{:#?}", (self.runner)()?);

        if !STORE.contains(&key)? {
            STORE.insert(key, value)?;
            return STORE.save();
        }

        STORE.compare(&key, |stored| match Diff::new(stored, &value) {
            Diff::Lines(ref diff) => panic!("{}", Report::new(&self, diff)),
            Diff::Same => Ok(()),
        })
    }
}

impl<T: Debug> Builder<T> {
    pub fn build(&mut self) -> Test<T> {
        Test {
            file: required("file", &mut self.file),
            ident: required("ident", &mut self.ident),
            module: required("module", &mut self.module),
            output: None,
            runner: required("runner", &mut self.runner),
            ret_ty: required("ret_ty", &mut self.ret_ty),
        }
    }

    pub fn file(&mut self, value: &'static str) -> &mut Builder<T> {
        self.file = Some(value);
        self
    }

    pub fn ident(&mut self, value: &'static str) -> &mut Builder<T> {
        self.ident = Some(value);
        self
    }

    pub fn module(&mut self, value: &'static str) -> &mut Builder<T> {
        self.module = Some(value);
        self
    }

    pub fn ret_ty(&mut self, value: &'static str) -> &mut Builder<T> {
        self.ret_ty = Some(value);
        self
    }

    pub fn runner(&mut self, value: Runner<T>) -> &mut Builder<T> {
        self.runner = Some(value);
        self
    }
}

impl<'a> Report<'a> {
    fn new<T>(test: &'a Test<T>, diff: &'a [Line<'a>]) -> Report<'a> {
        Report {
            diff,
            file: test.file,
            ident: test.ident,
            module: test.module,
            ret_ty: test.ret_ty,
        }
    }

    fn dirname(&self) -> Result<PathBuf, Error> {
        let mut path = PathBuf::new();
        let cwd = current_dir()?;

        path.push(self.file);

        for component in cwd.components() {
            let part = component.as_ref();

            if path.starts_with(&part) {
                path.strip_prefix(part)?;
            }
        }

        match path.parent() {
            Some(parent) => Ok(parent.to_owned()),
            None => Ok(PathBuf::new()),
        }
    }

    fn filename(&self) -> Option<String> {
        let mut path = PathBuf::new();

        path.push(self.file);
        Some(path.file_name()?.to_str()?.to_owned())
    }
}

impl<'a> Display for Report<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        macro_rules! indentln {
            ( $s:expr ) => ( indentln!($s,) );
            ( $s:expr, $($arg:expr),* ) => (
                writeln!(f, concat!("  ", $s), $($arg),*)
            );
        }

        macro_rules! newln {
            () => ( writeln!(f, "") );
        }

        newln!()?;
        newln!()?;
        newln!()?;
        indentln!(
            "{} {}{} {}{}{}",
            reverse!(red!(" FAILED ")),
            dimmed!("{}::", self.module),
            self.ident,
            dimmed!("({}/", self.dirname().unwrap().display()),
            self.filename().unwrap(),
            dimmed!(")")
        )?;
        newln!()?;
        indentln!("{}", green!("- Snapshot"))?;
        indentln!("{}", red!("+ Received"))?;

        newln!()?;
        indentln!(
            "{} {}() -> {} {{",
            purple!("fn"),
            blue!("{}", self.ident),
            purple!("{}", self.ret_ty)
        )?;
        newln!()?;

        for line in self.diff {
            match *line {
                Line::Eq(value) => indentln!("      {}", dimmed!("{}", value))?,
                Line::Ne(left, right) => {
                    indentln!("{}     {}", green!("-"), green!("{}", left))?;
                    indentln!("{}     {}", red!("+"), red!("{}", right))?;
                }
            }
        }

        newln!()?;
        indentln!("}}")?;
        newln!()?;
        newln!()
    }
}

fn required<T>(name: &str, opt: &mut Option<T>) -> T {
    match opt.take() {
        Some(value) => value,
        None => panic!("{} is a required field", name),
    }
}
