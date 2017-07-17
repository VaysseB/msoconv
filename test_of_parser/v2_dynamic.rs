use std::iter::Iterator;
use std::vec::IntoIter;
use std::collections::HashMap;
use std::rc::Rc;


#[derive(Debug, PartialEq)]
enum NextAction {
    Stay,
    ForwardOrStay(String),
    Done
}

#[derive(Debug)]
enum ParsedData {
    NoPreviousData,
    Root(Root),
    Li(Li)
}

struct Parser<'a> {
    source: &'a mut Iterator<Item=XmlEvent>,
    transitions: HashMap<&'static str, Vec<&'static str>>,
    states: HashMap<&'static str, Rc<Box<Fn(&mut Parser, ParsedData, ParsedData) -> (NextAction, ParsedData)>>>
}

impl<'a> Parser<'a> {
    fn new(source: &mut Iterator<Item=XmlEvent>) -> Parser {
        let mut parser = Parser {
            source: source,
            transitions: HashMap::new(),
            states: HashMap::new()
        };

        // for each set of transitions
        {
            let mut moves = Vec::new();
            moves.push(stringify!(li));
            parser.transitions.insert(stringify!(root), moves);
        }

        // for each state
        {
            let functor = Box::new(|ref mut _self: &mut Parser, mydata: ParsedData, last_forward: ParsedData| -> (NextAction, ParsedData) {
                #[allow(unused_mut)]
                let mut root = if
                    let ParsedData::Root(data) = mydata { data }
                else { Root::default() };
                let mut action = NextAction::Stay;

                // what to do after a forward parsing
                {
                    if let ParsedData::Li(li) = last_forward {
                        root.content.push(li);
                    }
                }

                // read the source
                while action == NextAction::Stay {
                    match _self.source.next() {
                        None => { action = NextAction::Done },
                        Some(element) => {
                            println!("In {}, got {:?}", stringify!(Root), element);

                            // parse of element
                            {
                            }

                            // get next state
                            action =
                                if let XmlEvent::StartElement { name } = element {
                                    NextAction::ForwardOrStay(name.clone())
                                } else {
                                    NextAction::Stay
                                };
                        }
                    }
                }

                // return parsed result of this state
                (action, ParsedData::Root(root))
            });

            parser.states.insert(stringify!(root), Rc::new(functor));
        }

        {
            #[allow(unused_variables)]
            let functor = Box::new(|ref mut _self: &mut Parser, mydata: ParsedData, last_forward: ParsedData| -> (NextAction, ParsedData) {
                #[allow(unused_mut)]
                let mut li = if
                    let ParsedData::Li(data) = mydata { data }
                else { Li::default() };
                let mut action = NextAction::Stay;

                // what to do after a forward parsing
                {
                }

                // read the source
                while action == NextAction::Stay {
                    match _self.source.next() {
                        None => { action = NextAction::Done },
                        Some(element) => {
                            println!("In {}, got {:?}", stringify!(Li), element);

                            // parse of element
                            {
                                match &element {
                                    &XmlEvent::CData(ref cdata) => li.text.push_str(cdata.as_str()),
                                    &XmlEvent::Characters(ref chars) => li.text.push_str(chars.as_str()),
                                    _ => ()
                                }
                            }

                            // find the next state if any
                            action =
                                if let XmlEvent::EndElement { name } = element {
                                    if name == stringify!(li) {
                                        NextAction::Done
                                    } else {
                                        NextAction::Stay
                                    }
                                } else {
                                    NextAction::Stay
                                };
                        }
                    }
                }

                // return parsed result of this state
                (action, ParsedData::Li(li))
            });

            parser.states.insert(stringify!(li), Rc::new(functor));
        }

        parser
    }

    fn parse(&mut self, entry_state_name: &'static str) -> ParsedData {
        let mut stack : Vec<(String, ParsedData)> = Vec::new();
        stack.push((entry_state_name.to_owned(), ParsedData::NoPreviousData));
        let mut last_data = ParsedData::NoPreviousData;

        while let Some(state) = stack.pop() {
            let parser = self.states.get(&state.0.as_str())
                .expect("state is registered").clone();
            let (action, updated_data) = parser(self, state.1, last_data);
            last_data = ParsedData::NoPreviousData;

            match action {
                NextAction::Stay => {
                    stack.push((state.0, updated_data));
                },
                NextAction::ForwardOrStay(next_state_name) => {
                    if self.allowed_move(&state.0, &next_state_name) {
                        stack.push((state.0, updated_data));
                        stack.push((next_state_name.to_owned(), ParsedData::NoPreviousData));
                    } else {
                        stack.push((state.0, updated_data));
                    }
                },
                NextAction::Done => {
                    last_data = updated_data;
                }
            }
        }

        last_data
    }

    fn allowed_move(&self, from_state: &String, to_state: &String) -> bool {
        if let Some(ref transition) = self.transitions.get(&from_state.as_str()) {
            transition.contains(&to_state.as_str())
        } else {
            false
        }
    }
}


#[derive(Debug, Default)]
struct Root {
    content: Vec<Li>
}

#[derive(Debug, Default)]
struct Li {
    text: String
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
    let raw_result = Parser::new(&mut source).parse("root");
    println!("Result: {:?}", raw_result);
}
