mod interactive;
mod simple;

use command::{CompilationCommand, JudgingCommand};
use config::Config;
use errors::{JudgeError, JudgeResult};
use palette::Palette;
use testsuite::{TestCase, TestCases};

use unicode_width::UnicodeWidthStr as _UnicodeWidthStr;

use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

/// Executes the tests.
///
/// # Errors
///
/// Returns `Err` if compilation or execution command fails, or any test fails.
pub(crate) fn judge(prop: JudgeProp) -> JudgeResult<()> {
    fn judge_all<C: TestCase, O: JudgingOutput>(
        cases: Vec<C>,
        solver: &Arc<JudgingCommand>,
        judge: fn(&C, &Arc<JudgingCommand>) -> JudgeResult<O>,
    ) -> JudgeResult<()> {
        let num_cases = cases.len();
        println_plural!("Running {}...", num_cases, "test", "tests");

        let filenames = cases.iter().map(|case| case.name()).collect::<Vec<_>>();
        let filename_max_width = filenames.iter().map(|s| s.width_cjk()).max().unwrap_or(0);
        let outputs = cases
            .into_iter()
            .zip(filenames.iter())
            .enumerate()
            .map(|(i, (case, filename))| {
                let output = judge(&case, solver)?;
                output.print_title(i, num_cases, &filename, filename_max_width);
                Ok(output)
            }).collect::<JudgeResult<Vec<O>>>()?;

        let num_failures = outputs.iter().filter(|o| o.failure()).count();
        if num_failures == 0 {
            println_plural!("All of the {} passed.", num_cases, "test", "tests");
            Ok(())
        } else {
            outputs
                .iter()
                .zip(filenames.iter())
                .enumerate()
                .for_each(|(i, (output, filename))| {
                    eprintln!("");
                    output.eprint_title(i, num_cases, filename, filename_max_width);
                    output.eprint_details();
                });
            Err(JudgeError::TestFailure(num_failures, num_cases))
        }
    }

    let cases = prop.cases;
    let case_paths = prop.case_paths;
    let solver = prop.solver;
    let solver_compilation = prop.solver_compilation;
    let tester_compilations = prop.tester_compilations;
    if let Some(solver_compilation) = solver_compilation {
        solver_compilation.execute()?;
        println!();
    }
    for tester_compilation in tester_compilations {
        tester_compilation.execute()?;
        println!();
    }
    solver.print_info(&case_paths);
    match cases {
        TestCases::Simple(cases) => judge_all(cases, &Arc::new(solver), simple::judge),
        TestCases::Interactive(cases) => judge_all(cases, &Arc::new(solver), interactive::judge),
    }
}

pub(crate) struct JudgeProp {
    cases: TestCases,
    case_paths: String,
    solver: JudgingCommand,
    solver_compilation: Option<CompilationCommand>,
    tester_compilations: HashSet<Arc<CompilationCommand>>,
}

impl JudgeProp {
    pub fn new(config: &Config, problem: &str, language: Option<&str>) -> ::Result<Self> {
        let (cases, paths) = config.suite_paths().load_merging(config, problem)?;
        let solver = config.solver(language)?.expand(&problem)?;
        let solver_compilation = match config.solver_compilation(language)? {
            Some(compilation) => Some(compilation.expand(&problem)?),
            None => None,
        };

        Ok(Self {
            tester_compilations: cases.interactive_tester_compilations(),
            cases,
            case_paths: paths,
            solver,
            solver_compilation,
        })
    }
}

pub(self) trait JudgingOutput
where
    Self: fmt::Display,
{
    fn failure(&self) -> bool;
    fn palette(&self) -> Palette;
    fn eprint_details(&self);

    fn print_title(&self, i: usize, n: usize, name: &str, name_width: usize) {
        (0..format!("{}", n).len() - format!("{}", i + 1).len()).for_each(|_| print!(" "));
        let msg = format!("{}/{} ({})", i + 1, n, name);
        print!("{}", Palette::Plain.bold().paint(msg));
        (0..name_width - name.width_cjk() + 1).for_each(|_| print!(" "));
        println!("{}", self.palette().bold().paint(self.to_string()));
    }

    fn eprint_title(&self, i: usize, n: usize, name: &str, name_width: usize) {
        (0..format!("{}", n).len() - format!("{}", i + 1).len()).for_each(|_| eprint!(" "));
        let msg = format!("{}/{} ({})", i + 1, n, name);
        eprint!("{}", Palette::Plain.bold().paint(msg));
        (0..name_width - name.width_cjk() + 1).for_each(|_| eprint!(" "));
        eprintln!("{}", self.palette().bold().paint(self.to_string()));
    }
}

pub(self) trait MillisRoundedUp {
    /// As milliseconds rounded up.
    fn millis_rounded_up(self) -> u64;
}

impl MillisRoundedUp for Duration {
    fn millis_rounded_up(self) -> u64 {
        (1_000_000_000 * self.as_secs() + u64::from(self.subsec_nanos()) + 999_999) / 1_000_000
    }
}
