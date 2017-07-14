extern crate argparse;


pub struct ArgOptions {
    pub filename: String
}

impl Default for ArgOptions {
    fn default() -> ArgOptions {
        ArgOptions { filename: "".to_owned() }
    }
}


pub fn parse() -> ArgOptions {
    let mut argopt = ArgOptions::default();

    // parsing of program parameters
    {
        let mut prog = argparse::ArgumentParser::new();
        prog.set_description("Convert MS Office document 'docx' to text.");
        prog.refer(&mut argopt.filename)
            .required()
            .metavar("file")
            .add_argument(
                "file",
                argparse::Store,
                "MS Office file");
        prog.parse_args_or_exit();
    }

    return argopt;
}
