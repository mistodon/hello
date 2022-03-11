use color_eyre::Result;
use structopt::StructOpt;

use hi::HelloArgs;

fn main() -> Result<()> {
    let args = HelloArgs::from_args();

    hi::report(&args)
}
