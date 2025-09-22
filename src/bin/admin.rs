use prontodb::lib::cli::admin::run_admin_cli;

fn main() {
    let exit_code = run_admin_cli();
    std::process::exit(exit_code);
}
