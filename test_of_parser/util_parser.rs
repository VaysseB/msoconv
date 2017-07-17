
#[derive(Debug, PartialEq)]
pub enum NextAction {
    Stay,
    ForwardOrStay(String),
    DoneIfStay(String),
    Done
}

pub trait ParserState<T, W> {
    fn read(&mut self, element: &T);

    fn after_forwards(&mut self, childre_data: &mut Vec<W>);
}

#[macro_export]
macro_rules! decl_parser {
    ( $parser:ident,
      item: $sitem:ident,
      dispatch: $dispatcher:ident,
      data wrapper: $dwrapper:ident,
      states: [ $($state_name:ident as $state_struct:ident),* ],
      phony: [ $($phony:ident),* ],
      transitions: $( $from_state:ident => [ $($to_state:ident),* ] ),*) => {

        use std::iter::Iterator;
        use std::vec::IntoIter;
        use std::collections::HashMap;
        use std::rc::Rc;

        #[derive(Debug)]
        enum $dwrapper {
            NoPreviousData,
            $( $state_struct($state_struct), )*
        }

        struct $parser<'a> {
            source: &'a mut Iterator<Item=$sitem>,
            transitions: HashMap<&'static str, Vec<&'static str>>,
            states: HashMap<&'static str, Rc<Box<Fn(&mut $parser, &mut Vec<$dwrapper>) -> NextAction>>>,
            debug: bool
        }

        impl<'a> $parser<'a> {
            fn new(source: &mut Iterator<Item=$sitem>) -> $parser {
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
                    let functor = Box::new(|ref mut _self: &mut $parser, collected_data: &mut Vec<$dwrapper>| -> NextAction {
                        let mut state = match collected_data.pop() {
                            None => $state_struct::default(),
                            Some(mydata) => {
                                match mydata {
                                    $dwrapper::$state_struct(mydata) => mydata,
                                    _ => $state_struct::default()
                                }
                            }
                        };
                        let mut action = NextAction::Stay;

                        // what to do after a forward parsing
                        state.after_forwards(collected_data);

                        // read the source
                        while action == NextAction::Stay {
                            match _self.source.next() {
                                None => { action = NextAction::Done },
                                Some(element) => {
                                    if _self.debug {
                                        println!("In {}, got {:?}", stringify!($state_name), element);
                                    }

                                    // parse of element
                                    state.read(&element);

                                    // get next state
                                    action = $dispatcher(element);
                                }
                            }
                        }

                        collected_data.push($dwrapper::$state_struct(state));

                        // return parsed result of this state
                        action
                    });

                    parser.states.insert(stringify!($state_name), Rc::new(functor));
                })*

                // for each phony state
                $({
                    let functor = Box::new(|ref mut _self: &mut $parser, _: &mut Vec<$dwrapper>| -> NextAction {
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
                        action
                    });

                    parser.states.insert(stringify!($phony), Rc::new(functor));
                })*

                parser
            }

            fn parse(&mut self, entry_state_name: &'static str) -> Vec<$dwrapper> {
                let mut stack : Vec<(String, $dwrapper)> = Vec::new();
                stack.push((entry_state_name.to_owned(), $dwrapper::NoPreviousData));
                let mut collected_data = Vec::new();

                while let Some(state) = stack.pop() {
                    let parser = self.states.get(&state.0.as_str())
                        .expect("state is registered").clone();
                    collected_data.push(state.1);
                    let action = parser(self, &mut collected_data);
                    let state_data = collected_data
                        .pop()
                        .expect("state miss is own data");

                    match action {
                        NextAction::Stay => {
                            stack.push((state.0, state_data));
                        },
                        NextAction::ForwardOrStay(next_state_name) => {
                            if self.allowed_move(&state.0, &next_state_name) {
                                stack.push((state.0, state_data));
                                stack.push((next_state_name.to_owned(), $dwrapper::NoPreviousData));
                            } else {
                                stack.push((state.0, state_data));
                            }
                        },
                        NextAction::DoneIfStay(same_state_name) => {
                            if same_state_name == state.0 {
                                collected_data.push(state_data);
                            } else {
                                stack.push((state.0, state_data));
                            }
                        },
                        NextAction::Done => {
                            collected_data.push(state_data);
                        }
                    }
                }

                collected_data
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

