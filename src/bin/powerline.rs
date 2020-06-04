extern crate powerline;

use powerline::{modules::*, theme::SimpleTheme};

#[cfg(feature = "time")]
use powerline::modules::Time;

fn main() -> powerline::R<()> {
	let mut prompt = powerline::Powerline::new();

	//#[cfg(feature = "time")]
	//prompt.add_module(Time::<SimpleTheme>::with_time_format("%H:%M:%S"))?;

	//prompt.add_module(User::<SimpleTheme>::new())?;
	//prompt.add_module(Host::<SimpleTheme>::new())?;
	prompt.add_module(Cwd::<SimpleTheme>::new(20, 2, false))?;
	prompt.add_module(Git::<SimpleTheme>::new())?;
	prompt.add_module(ReadOnly::<SimpleTheme>::new())?;
	prompt.add_module(Cmd::<SimpleTheme>::new())?;
	// prompt.add_module(ExitCode::<SimpleTheme>::new())?;

	println!("{}", prompt);
	Ok(())
}
