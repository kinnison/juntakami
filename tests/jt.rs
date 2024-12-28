use juntakami_steps::subplotlib::{self, prelude::*};

juntakami_steps::jt_binary_on_path!(env!("CARGO_BIN_EXE_juntakami"));

subplotlib::codegen!("steps/juntakami.subplot");
