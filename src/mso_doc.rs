extern crate zip;
extern crate xml;

use std::io;
use std::fs;
use std::iter::Iterator;

use self::xml::reader::XmlEvent;
use self::xml::name::OwnedName;
use self::xml::attribute::OwnedAttribute;


pub trait WordReader {
    fn paragraph_row(&mut self, text: &String, style: &String);

    fn table_header(&mut self, columns: &Vec<String>, style: &String);
    //    fn table_row(&mut self, columns: &Vec<String>, style: &String);
}


mod sax_docx {
    use mso_doc::*;

    trait UtilsName {
        fn is_tag(&self, key: &str) -> bool;
    }

    impl UtilsName for OwnedName {
        fn is_tag(&self, key: &str) -> bool {
            if let Some(i) = key.find(":") {
                let (namespace, tail) = key.split_at(i);
                let (_, key) = tail.split_at(1);

                if let Some(ref p) = self.prefix {
                    p.as_str() == namespace && self.local_name.as_str() == key
                } else {
                    false
                }
            } else {
                if let None = self.prefix {
                    self.local_name.as_str() == key
                } else {
                    false
                }
            }
        }
    }

    trait UtilsAttributes {
        fn value(&self, key: &str) -> String;
    }

    impl UtilsAttributes for Vec<OwnedAttribute> {
        fn value(&self, key: &str) -> String {
            self.iter()
                .skip_while(|attr| !attr.name.is_tag(key))
                .next()
                .map(|attr| attr.value.to_owned())
                .unwrap_or_else(String::new)
        }
    }

    #[derive(Default)]
    pub struct DocumentRoot {
        paragraph: Paragraph,
        //table: Table
    }

    impl DocumentRoot {
        pub fn parse<T>(&mut self, source: &mut T, reader: &mut WordReader)
            where T: Iterator<Item=Result<XmlEvent, xml::reader::Error>> {
                while let Some(event) = source.next() {
                    match event.unwrap() {
                        XmlEvent::StartElement { ref name, .. } => {
                            if Paragraph::is_tag(name) {
                                self.paragraph.parse(source);
                                self.paragraph.send(reader);
                                //} else if Table::is_tag(name) {
                                //self.table.parse_send(source, reader);
                        }
                        },
                        _ => ()
                    }
                }
            }
    }

    // Paragraph
    // <w:p>
    //   - settings
    //   - content
    #[derive(Default)]
    struct Paragraph {
        style: RefToStyle,
        text: Text,
    }

    impl Paragraph {
        fn is_tag(name: &OwnedName) -> bool {
            if let Some(ref p) = name.prefix {
                p.as_str() == "w" && name.local_name == "p"
            } else {
                false
            }
        }

        fn send(&mut self, reader: &mut WordReader) {
            reader.paragraph_row(&self.text.content, &self.style.name);
            self.style.clear();
            self.text.clear();
        }

        pub fn parse<T>(&mut self, source: &mut T)
            where T: Iterator<Item=Result<XmlEvent, xml::reader::Error>> {
                while let Some(event) = source.next() {
                    match event.unwrap() {
                        XmlEvent::StartElement { ref name, ref attributes, .. } => {
                            if Self::is_tag(name) {
                                panic!(format!("Nested paragraph not supported"));
                            } else if RefToStyle::is_tag(name) {
                                self.style.parse(source, attributes);
                            } else if Text::is_tag(name) {
                                self.text.parse(source);
                            }
                        },
                        XmlEvent::EndElement { ref name, .. }
                        if Self::is_tag(name) => break,
                        _ => ()
                    }
                }
            }
    }

    // RefToStyle
    // <w:pStyle w:val="style">
    #[derive(Default)]
    struct RefToStyle {
        name: String
    }

    impl RefToStyle {
        fn is_tag(name: &OwnedName) -> bool {
            if let Some(ref p) = name.prefix {
                p.as_str() == "w" && name.local_name == "pStyle"
            } else {
                false
            }
        }

        fn clear(&mut self) {
            self.name.clear();
        }

        pub fn parse<T>(&mut self, source: &mut T, attributes: &Vec<OwnedAttribute>)
            where T: Iterator<Item=Result<XmlEvent, xml::reader::Error>> {
                self.name = attributes.value("w:val");

                while let Some(event) = source.next() {
                    match event.unwrap() {
                        XmlEvent::StartElement { ref name, .. }
                        if Self::is_tag(name) => panic!(format!("Nested style definition not supported")),
                        XmlEvent::EndElement { ref name, .. }
                        if Self::is_tag(name) => break,
                        _ => ()
                    }
                }
            }
    }

    // Text
    // <w:t>text</w:t>
    #[derive(Default)]
    struct Text {
        content: String
    }

    impl Text {
        fn is_tag(name: &OwnedName) -> bool {
            if let Some(ref p) = name.prefix {
                p.as_str() == "w" && name.local_name == "t"
            } else {
                false
            }
        }

        fn clear(&mut self) {
            self.content.clear();
        }

        pub fn parse<T>(&mut self, source: &mut T)
            where T: Iterator<Item=Result<XmlEvent, xml::reader::Error>> {
                while let Some(event) = source.next() {
                    match event.unwrap() {
                        XmlEvent::StartElement { ref name, .. }
                        if Self::is_tag(name) => panic!(format!("Nested text not supported")),
                        XmlEvent::CData(ref cdata) => self.content.push_str(cdata),
                        XmlEvent::Characters(ref chars) => self.content.push_str(chars),
                        XmlEvent::Whitespace(ref whsp) => self.content.push_str(whsp),
                        XmlEvent::EndElement { ref name, .. }
                        if Self::is_tag(name) => break,
                        _ => ()
                    }
                }
            }
    }

    /*    // Table*/
    //// <w:tbl>
    ////   - row
    //#[derive(Default)]
    //struct Table {
    //row: TableRow,
    //}

    //impl Table {
    //fn is_tag(name: &OwnedName) -> bool {
    //if let Some(ref p) = name.prefix {
    //p.as_str() == "w" && name.local_name == "tbl"
    //} else {
    //false
    //}
    //}

    //pub fn parse_send<T>(&mut self, source: &mut T, reader: &WordReader)
    //where T: Iterator<Item=Result<XmlEvent, xml::reader::Error>> {
    //while let Some(event) = source.next() {
    //match event.unwrap() {
    //XmlEvent::StartElement { ref name, .. } => {
    //if Self::is_tag(name) {
    //panic!(format!("Nested table not supported"));
    //} else if Text::is_tag(name) {
    //self.row.parse(source);
    //self.row.send(reader);
    //}
    //},
    //XmlEvent::EndElement { ref name, .. }
    //if Self::is_tag(name) => break,
    //_ => ()
    //}
    //}
    //}
    /*}*/
}


pub fn parse(filepath: &String, reader: &mut WordReader) {
    let file = fs::File::open(&filepath).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let word_doc_file = archive.by_name("word/document.xml").unwrap();
    let buffer = io::BufReader::new(word_doc_file);
    let xml_parser = xml::reader::EventReader::new(buffer);
    let mut root = sax_docx::DocumentRoot::default();
    root.parse(&mut xml_parser.into_iter(), reader);
}
