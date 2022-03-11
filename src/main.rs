use color_eyre::Result;
use structopt::StructOpt;

use hello::HelloArgs;

fn main() -> Result<()> {
    let args = HelloArgs::from_args();

    hello::report(&args)
}
