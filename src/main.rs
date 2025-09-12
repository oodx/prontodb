
// Use RSB prelude for macros (bootstrap!/pre_dispatch!/dispatch!)
use rsb::prelude::*;
use prontodb::core::dispatch::pronto_dispatch;

//use std::process::exit;


//use rsb::deps::rand::{Rng};

fn main() -> i32 {

  //sanity check
  info!("Loading ProntoDB main");

  //bootstrap
  let args = bootstrap!();
  info!("Args received: {:?}", args);

  //options

  //address

  //pre-dispatch

  //dispatch
  let exit_code = pronto_dispatch(args);


  exit_code

}



