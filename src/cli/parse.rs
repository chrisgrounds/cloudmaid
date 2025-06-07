use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
  #[arg(short, long)]
  pub input_file: String,

  #[arg(short, long)]
  pub output_file: String,
}
