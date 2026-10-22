fn main() {
    match gesa_cli::update::launcher_main() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("GESA launcher: {error}");
            std::process::exit(1);
        }
    }
}
