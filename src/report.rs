use std::fmt::{self, Display, Formatter};

use test::{Outcome, Test};

/// Meta information about a test and test result.
#[derive(Debug)]
pub struct Report {
    outcome: Outcome,
    test: Test,
}

impl Report {
    pub fn new(test: Test, outcome: Outcome) -> Report {
        Report { outcome, test }
    }

    pub fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    fn display_failure(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(
            f,
            "  {} {}{} {}{}{}",
            reverse!(red!("  FAILED  ")),
            dimmed!("{}::", self.test.path()),
            self.test.name(),
            dimmed!("({}/", self.test.dirname().unwrap().display()),
            self.test.basename().unwrap().display(),
            dimmed!(")")
        )?;
        writeln!(f, "")?;
        writeln!(f, "  {}", green!("- Snapshot"))?;
        writeln!(f, "  {}", red!("+ Received"))?;
        writeln!(f, "")?;
        writeln!(
            f,
            "    {} {}() -> {} {{",
            purple!("fn"),
            blue!("{}", self.test.name()),
            purple!("{}", self.test.ret())
        )?;
        writeln!(f, "")?;

        for line in &self.outcome.diff() {
            use diff::Result::*;

            match *line {
                Left(l) => writeln!(f, "  {}     {}", green!("-"), green!("{}", l))?,
                Right(r) => writeln!(f, "  {}     {}", red!("+"), red!("{}", r))?,
                Both(_, r) => writeln!(f, "        {}", dimmed!("{}", r))?,
            }
        }

        writeln!(f, "")?;
        writeln!(f, "    }}")
    }

    fn display_success(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(
            f,
            "  {} {}{} {}{}{}",
            reverse!(green!("  PASSED  ")),
            dimmed!("{}::", self.test.path()),
            self.test.name(),
            dimmed!("({}/", self.test.dirname().unwrap().display()),
            self.test.basename().unwrap().display(),
            dimmed!(")")
        )
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "")?;
        writeln!(f, "")?;
        writeln!(f, "")?;

        if self.outcome.was_successful() {
            self.display_success(f)?;
        } else {
            self.display_failure(f)?;
        }

        writeln!(f, "")?;
        writeln!(f, "")
    }
}
