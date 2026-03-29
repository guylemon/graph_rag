use std::process::exit;

use graph_rag::run;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        exit(1)
    }
}
