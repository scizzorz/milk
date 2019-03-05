use exitfailure::ExitFailure;
use milk::run_supercommand;

fn main() -> Result<(), ExitFailure> {
  env_logger::init();
  run_supercommand("milk-branch")
}
