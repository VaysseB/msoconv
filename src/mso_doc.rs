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
}


mod sax_docx {
    use mso_doc::*;

    #[derive(Default)]
    pub struct DocumentRoot {
        paragraph: Paragraph
    }

    impl DocumentRoot {
        pub fn parse<T>(&mut self, source: &mut T, reader: &mut WordReader)
            where T: Iterator<Item=Result<XmlEvent, xml::reader::Error>> {
                while let Some(event) = source.next() {
                    match event.unwrap() {
                        XmlEvent::StartElement { ref name, .. } => {
                            if Paragraph::is_tag(name) {
                                self.paragraph.parse(source);
                                self.send_paragraph(reader);
                            }
                        },
                        _ => ()
                    }
                }
            }

        fn send_paragraph(&mut self, reader: &mut WordReader) {
            reader.paragraph_row(
                &self.paragraph.text.content,
                &self.paragraph.style.name
                );
            self.paragraph.clear();
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

        fn clear(&mut self) {
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

        fn is_style_name_key(name: &OwnedName) -> bool {
            if let Some(ref p) = name.prefix {
                p.as_str() == "w" && name.local_name == "val"
            } else {
                false
            }
        }

        fn clear(&mut self) {
            self.name.clear();
        }

        pub fn parse<T>(&mut self, source: &mut T, attributes: &Vec<OwnedAttribute>)
            where T: Iterator<Item=Result<XmlEvent, xml::reader::Error>> {
                self.name = attributes.iter()
                    .skip_while(|attr| !Self::is_style_name_key(&attr.name))
                    .next()
                    .map(|attr| attr.value.to_owned())
                    .unwrap_or_else(String::new);

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
