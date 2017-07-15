
mod parameters;
mod mso_doc;


struct Converter {
    started_table_row: bool,
    column_i: u32
}

impl Default for Converter {
    fn default() -> Converter {
        Converter {
            started_table_row: false,
            column_i: 0
        }
    }
}

impl mso_doc::WordReader for Converter {
    fn paragraph_row(&mut self, text: &String, style: &String) {
        println!("{}\t{}", style, text);
    }

    fn table_new_row(&mut self) {
        if self.started_table_row {
            println!("");
        }
        self.column_i = 0;
        self.started_table_row = false;
    }

    fn table_closed(&mut self) {
        if self.started_table_row {
            println!("");
        }
    }

    fn table_cell(&mut self, text: &String, style: &String, header: bool) {
        self.started_table_row = true;
        self.column_i += 1;
        print!("|{}{} {}\t{}",
               if header { "#" } else { "" },
               self.column_i,
               style,
               text);
    }
}


fn main() {
    let params = parameters::parse();
    println!("Input: {}", params.filename);
    // TODO deal with all errors here: file doesn't exist, bad file format, bad xml structure with
    // making your head burn with Rust error handling
    mso_doc::parse(&params.filename, &mut Converter::default());
}
