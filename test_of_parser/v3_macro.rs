use std::iter::Iterator;
use std::vec::IntoIter;
use std::collections::HashMap;
use std::rc::Rc;


#[derive(Debug, PartialEq)]
enum NextAction {
    Stay,
    ForwardOrStay(String),
    DoneIfStay(String),
    Done
}

trait ParserState<T, W> {
    fn read(&mut self, element: &T);

    fn after_forward(&mut self, child_data: W);
}

macro_rules! decl_parser {
    ( $parser:ident, item: $src_item:ident, dispatch: $dispatcher:ident, data wrapper: $data_wrapper:ident, states: [ $($state_name:ident as $state_struct:ident),* ], phony: [ $($phony:ident),* ], transitions: $( $from_state:ident => [ $($to_state:ident),* ] ),*) => {

        #[derive(Debug)]
        enum $data_wrapper {
            NoPreviousData,
            $( $state_struct($state_struct), )*
        }

        struct $parser<'a> {
            source: &'a mut Iterator<Item=$src_item>,
            transitions: HashMap<&'static str, Vec<&'static str>>,
            states: HashMap<&'static str, Rc<Box<Fn(&mut $parser, $data_wrapper, $data_wrapper) -> (NextAction, $data_wrapper)>>>,
            debug: bool
        }

        impl<'a> $parser<'a> {
            fn new(source: &mut Iterator<Item=$src_item>) -> $parser {
                let mut parser = $parser {
                    source: source,
                    transitions: HashMap::new(),
                    states: HashMap::new(),
                    debug: false
                };

                // for each set of transitions
                $({
                    let mut moves = Vec::new();
                    $(
                        moves.push(stringify!($to_state));
                     )* //;
                    parser.transitions.insert(stringify!($from_state), moves);
                })*

                // for each state used by the user
                $({
                    let functor = Box::new(|ref mut _self: &mut $parser, mydata: $data_wrapper, last_forward: $data_wrapper| -> (NextAction, $data_wrapper) {
                        #[allow(unused_mut)]
                        let mut $state_name = if
                            let $data_wrapper::$state_struct(data) = mydata { data }
                        else { $state_struct::default() };
                        let mut action = NextAction::Stay;

                        // what to do after a forward parsing
                        $state_name.after_forward(last_forward);

                        // read the source
                        while action == NextAction::Stay {
                            match _self.source.next() {
                                None => { action = NextAction::Done },
                                Some(element) => {
                                    if _self.debug {
                                        println!("In {}, got {:?}", stringify!($state_name), element);
                                    }

                                    // parse of element
                                    $state_name.read(&element);

                                    // get next state
                                    action = $dispatcher(element);
                                }
                            }
                        }

                        // return parsed result of this state
                        (action, $data_wrapper::$state_struct($state_name))
                    });

                    parser.states.insert(stringify!($state_name), Rc::new(functor));
                })*

                // for each phony state
                $({
                    let functor = Box::new(|ref mut _self: &mut $parser, _: $data_wrapper, _: $data_wrapper| -> (NextAction, $data_wrapper) {
                        let mut action = NextAction::Stay;

                        // read the source
                        while action == NextAction::Stay {
                            match _self.source.next() {
                                None => { action = NextAction::Done },
                                Some(element) => {
                                    if _self.debug {
                                        println!("In {}, got {:?}", stringify!($phony), element);
                                    }

                                    // get next state
                                    action = $dispatcher(element);
                                }
                            }
                        }

                        // return parsed result of this state
                        (action, $data_wrapper::NoPreviousData)
                    });

                    parser.states.insert(stringify!($phony), Rc::new(functor));
                })*

                parser
            }

            fn parse(&mut self, entry_state_name: &'static str) -> $data_wrapper {
                let mut stack : Vec<(String, $data_wrapper)> = Vec::new();
                stack.push((entry_state_name.to_owned(), $data_wrapper::NoPreviousData));
                let mut last_data = $data_wrapper::NoPreviousData;

                while let Some(state) = stack.pop() {
                    let parser = self.states.get(&state.0.as_str())
                        .expect("state is registered").clone();
                    let (action, updated_data) = parser(self, state.1, last_data);
                    last_data = $data_wrapper::NoPreviousData;

                    match action {
                        NextAction::Stay => {
                            stack.push((state.0, updated_data));
                        },
                        NextAction::ForwardOrStay(next_state_name) => {
                            if self.allowed_move(&state.0, &next_state_name) {
                                stack.push((state.0, updated_data));
                                stack.push((next_state_name.to_owned(), $data_wrapper::NoPreviousData));
                            } else {
                                stack.push((state.0, updated_data));
                            }
                        },
                        NextAction::DoneIfStay(same_state_name) => {
                            if same_state_name == state.0 {
                                last_data = updated_data;
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
                    if self.debug {
                        if !self.states.contains_key(&to_state.as_str()) {
                            println!("Unregistered state {}", &to_state);
                        } else {
                            println!("Transition not allowed from {} to {}",
                                     &from_state, &to_state);
                        }
                    }
                    false
                }
            }
        }
    }
}

decl_parser!(
    Parser,
    item: XmlEvent,
    dispatch: get_next_state,
    data wrapper: ParsedData,
    states: [root as Root, li as Li],
    phony: [ul, title],
    transitions:
    root => [title, ul],
    ul => [li]
    );

//======================================

#[derive(Debug, Default)]
struct Root {
    content: Vec<Li>
}

#[derive(Debug, Default)]
struct Li {
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

    fn after_forward(&mut self, child_data: ParsedData) {
        if let ParsedData::Li(li) = child_data {
            self.content.push(li);
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

    fn after_forward(&mut self, _: ParsedData) {
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
    let mut reader = Parser::new(&mut source);
    reader.debug = false;
    println!("Result: {:?}", reader.parse("root"));
}
