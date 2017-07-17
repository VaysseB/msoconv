use std::iter::Iterator;
use std::vec::IntoIter;

#[derive(Debug, Default)]
struct Root {
    content: Vec<Li>
}

#[derive(Debug, Default)]
struct Li {
    text: String
}

struct Parser<'a> {
    source: &'a mut Iterator<Item=XmlEvent>
}

impl<'a> Parser<'a> {
    fn new(source: &mut Iterator<Item=XmlEvent>) -> Parser {
        Parser {
            source: source
        }
    }

    fn parse_root(&mut self) -> Root {
            let mut data = Root::default();
            while let Some(event) = self.source.next() {
                println!("From 'root': {:?}", &event);
                match event {
                    XmlEvent::StartElement { ref name } => {
                        if name.as_str() == "li" {
                            let res_li = self.parse_li();
                            data.content.push(res_li);
                        }
                    },
                    _ => ()
                }
            }
            data
        }

    fn parse_li(&mut self) -> Li {
            let mut li = Li::default();
            while let Some(event) = self.source.next() {
                println!("From 'li': {:?}", &event);
                match event {
                    XmlEvent::Characters(data) => {
                        println!("LI: {}", data);
                        li.text += data.as_str();
                    },
                    XmlEvent::EndElement { ref name }
                    if name.as_str() == "li" => break,
                    _ => ()
                }
            }
            li
        }
}


//======================================

#[derive(Debug)]
enum XmlEvent {
    StartElement { name: String },
    EndElement { name: String },
    Characters(String),
    CData(String)
}

fn main() {
    let input = vec![
        XmlEvent::StartElement { name: "html".to_owned() },
        XmlEvent::StartElement { name: "head".to_owned() },
        XmlEvent::StartElement { name: "title".to_owned() },
        XmlEvent::Characters("this is my title".to_owned()),
        XmlEvent::EndElement { name: "title".to_owned() },
        XmlEvent::EndElement { name: "head".to_owned() },
        XmlEvent::StartElement { name: "body".to_owned() },
        XmlEvent::StartElement { name: "div".to_owned() },
        XmlEvent::StartElement { name: "ul".to_owned() },
        XmlEvent::StartElement { name: "li".to_owned() },
        XmlEvent::Characters("List item A".to_owned()),
        XmlEvent::EndElement { name: "li".to_owned() },
        XmlEvent::StartElement { name: "li".to_owned() },
        XmlEvent::Characters("List item B".to_owned()),
        XmlEvent::Characters("on the same B list".to_owned()),
        XmlEvent::EndElement { name: "li".to_owned() },
        XmlEvent::EndElement { name: "ul".to_owned() },
        XmlEvent::EndElement { name: "div".to_owned() },
        XmlEvent::StartElement { name: "footer".to_owned() },
        XmlEvent::CData("Copyright:".to_owned()),
        XmlEvent::EndElement { name: "footer".to_owned() },
        XmlEvent::EndElement { name: "body".to_owned() },
        XmlEvent::EndElement { name: "html".to_owned() }
    ];

    let mut source : IntoIter<XmlEvent> = input.into_iter();
    let result = Parser::new(&mut source).parse_root();
    println!("Result: {:?}", result);
}
