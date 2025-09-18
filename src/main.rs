
use rsb::prelude::*;

// Keep the ProntoDB router central (uses RSB dispatch! under the hood)
use prontodb::core::dispatch::pronto_dispatch;

fn main() {
    // CLI + Host bootstrap, then parse options into Global
    let args = bootstrap!();
    options!(&args);

    // Route using RSB dispatch! inside pronto_dispatch (exits with handler status)
    pronto_dispatch(args);
}
