use crate::{shell::Shell, web::LazyLockedFile};
use snowchains_core::web::{Atcoder, Login, PlatformVariant};
use std::{
    io::{self, BufRead, Write},
    sync::{Arc, Mutex},
};
use structopt::StructOpt;
use strum::VariantNames as _;
use termcolor::WriteColor;

#[derive(StructOpt, Debug)]
pub struct OptLogin {
    /// Prints the output as a JSON value
    #[structopt(long)]
    pub json: bool,

    /// Coloring
    #[structopt(
        long,
        possible_values(crate::ColorChoice::VARIANTS),
        default_value("auto")
    )]
    pub color: crate::ColorChoice,

    /// Target platform
    #[structopt(possible_values(&["atcoder"]))]
    pub service: PlatformVariant,
}

pub(crate) fn run(
    opt: OptLogin,
    ctx: crate::Context<impl BufRead, impl Write, impl WriteColor>,
) -> anyhow::Result<()> {
    let OptLogin {
        json,
        color: _,
        service,
    } = opt;

    let crate::Context {
        cwd: _,
        mut stdin,
        mut stdout,
        stderr,
        draw_progress: _,
    } = ctx;

    let cookies_path = crate::web::cookies_path()?;
    let cookies_file = LazyLockedFile::new(&cookies_path);

    let cookie_store = crate::web::load_cookie_store(cookies_file.path())?;
    let on_update_cookie_store =
        |cookie_store: &_| crate::web::save_cookie_store(cookie_store, &cookies_file);

    let shell = Shell::new(&Arc::new(Mutex::new(io::empty())), stderr, false);

    let username_and_password = || -> _ {
        eprint!("Username: ");
        io::stderr().flush()?;
        let username = rpassword::read_password_with_reader(Some(&mut stdin))?;

        eprint!("Password: ");
        io::stderr().flush()?;
        let password = rpassword::read_password_with_reader(Some(&mut stdin))?;

        Ok((username, password))
    };

    debug_assert_eq!(service, PlatformVariant::Atcoder);

    let outcome = Atcoder::exec(Login {
        timeout: Some(crate::web::SESSION_TIMEOUT),
        cookie_store: (cookie_store, on_update_cookie_store),
        shell,
        credentials: (username_and_password,),
    })?;

    let message = if json {
        outcome.to_json()
    } else {
        match outcome {
            snowchains_core::web::LoginOutcome::Success => "Successfully logged in.",
            snowchains_core::web::LoginOutcome::AlreadyLoggedIn => "Already logged in.",
        }
        .to_owned()
    };

    writeln!(stdout, "{}", message)?;
    stdout.flush().map_err(Into::into)
}
