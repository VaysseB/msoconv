#[macro_use]
mod util_parser;
use util_parser::{NextAction, ParserState};

decl_parser!(
    Parser,
    item: XmlEvent,
    dispatch: get_next_state,
    data wrapper: ParsedData,
    states: [root as Root, li as Li],
    phony: [ul, title, probe],
    transitions:
    root => [title, ul],
    ul => [li],
    li => [probe]
    );

//======================================

#[derive(Debug, Default)]
pub struct Root {
    content: Vec<Li>
}

#[derive(Debug, Default)]
pub struct Li {
    text: String
}

fn get_next_state(event: XmlEvent) -> NextAction {
    match event {
        XmlEvent::StartElement { name } => NextAction::ForwardOrStay(name.clone()),
        XmlEvent::EndElement { name } => NextAction::DoneIfStay(name.clone()),
        _ => NextAction::Stay
    }
}

impl ParserState<XmlEvent, ParsedData> for Root {
    fn read(&mut self, _: &XmlEvent) {
    }

    fn after_forwards(&mut self, children_data: &mut Vec<ParsedData>) {
        while let Some(child) = children_data.pop() {
            println!("Done child of root {:?}", &child);
            if let ParsedData::Li(li) = child {
                self.content.push(li);
            }
        }
    }
}

impl ParserState<XmlEvent, ParsedData> for Li {
    fn read(&mut self, event: &XmlEvent) {
        match event {
            &XmlEvent::CData(ref cdata) => {
                self.text.push_str(&cdata.as_str());
            },
            &XmlEvent::Characters(ref chars) => {
                self.text.push_str(&chars.as_str());
            },
            _ => ()
        }
    }

    fn after_forwards(&mut self, _: &mut Vec<ParsedData>) {
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
        XmlEvent::StartElement { name: "probe".to_owned() },
        XmlEvent::EndElement { name: "probe".to_owned() },
        XmlEvent::StartElement { name: "probe".to_owned() },
        XmlEvent::EndElement { name: "probe".to_owned() },
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
    let mut reader = Parser::new(&mut source);
    reader.debug = true;
    println!("Result: {:?}", reader.parse("root"));
}
