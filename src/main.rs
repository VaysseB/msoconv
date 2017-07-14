
mod parameters;
mod mso_doc;


struct Converter {
}

impl mso_doc::WordReader for Converter {
    fn paragraph_row(&mut self, text: &String, style: &String) {
        println!("|{}\t{}", style, text);
    }
}


fn main() {
    let params = parameters::parse();
    println!("Input: {}", params.filename);
    // TODO deal with all errors here: file doesn't exist, bad file format, bad xml structure with
    // making your head burn with Rust error handling
    mso_doc::parse(&params.filename, &mut Converter{});
}
