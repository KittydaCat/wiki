// TODO
// Expand templates :(
// Space infront of lines make it weird
// Add wiki table support :(

use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::str::{Chars};
use serde_json::Value;
use either::Either;
use regex::Regex;

#[derive(Debug,Clone)]
enum Token {
    Text(String),
    Link(Link),
    Template(Template),
    Span(Span),
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Text(x) => {todo!()}
            Token::Link(x) => {todo!()}
            Token::Template(x) => {todo!()}
            Token::Span(x) => {todo!()}
        }
    }
}

#[derive(Clone, Debug)]
struct Link {
    article_name: String,
    display_name: String,
}

#[derive(Clone, Debug)]
struct Template {
    template_name: String,
    options: Vec<(Either<u32, String>, Vec<Token>)>
}

#[derive(Clone, Debug)]
struct Span {
    span_type: String,
    options: Vec<(String, String)>,
    inner_text: Vec<Token>
}

#[derive(Debug, Clone, PartialEq)]
enum ParserState {
    OpenBracket,
    ClosedBracket,
    OpenBrace,
    ClosedBrace,
    None,
}

fn parse(chars: &mut Chars) -> (Vec<Token>, Option<char>) {

    let mut state = ParserState::None;

    let mut tokens = Vec::new();

    let mut tempstr = String::new();

    while let Some(grabbed_char) = chars.next() {

        state = match (grabbed_char, state) {

            ('{', ParserState::None) => ParserState::OpenBrace,
            ('[', ParserState::None) => ParserState::OpenBracket,
            ('}', ParserState::None) => ParserState::ClosedBrace,
            (']', ParserState::None) => ParserState::ClosedBracket,

            ('[', ParserState::OpenBracket) => {

                // [[Plan 9 from Bell Labs|Plan 9]]

                tokens.push(Token::Text(tempstr));

                tempstr = String::new();

                tokens.push(Token::Link(parse_link(chars)));

                ParserState::None
            },

            ('{', ParserState::OpenBrace) => {

                // start parsing the template

                tokens.push(Token::Text(tempstr));

                tempstr = String::new();

                tokens.push(Token::Template(parse_template(chars)));

                ParserState::None
            },

            ('|', ParserState::OpenBrace) => {

                // wikitable

                tokens.push(Token::Text(tempstr));

                tempstr = String::new();

                let mut char2 = chars.next();

                loop {

                    let char1 = chars.next();

                    match (char2, char1) {
                        (None, None) => panic!(),
                        (Some(_), None) => panic!(),
                        (Some('|'), Some('}')) => {break;}
                        (Some(_), Some(_)) => {},
                        x => {
                            dbg!(x);
                            panic!()
                        }
                    }

                    char2 = char1;

                }

                ParserState::None
            },

            (']', ParserState::ClosedBracket) => {
                tokens.push(Token::Text(tempstr));

                return (tokens, Some(']'));
            }
            ('}', ParserState::ClosedBrace) => {
                tokens.push(Token::Text(tempstr));

                return (tokens, Some('}'));
            }

            ('<', ParserState::None) => {

                tokens.push(Token::Text(tempstr));

                tempstr = String::new();

                let x = chars.next().unwrap();

                // if its a closing tag its not our problem
                if x == '/' {
                    return (tokens, Some('/'));
                }

                tokens.push(Token::Span(parse_span(chars, x)));

                ParserState::None
            },

            ('|', ref x) => {

                assert_eq!(*x, ParserState::None);

                tokens.push(Token::Text(tempstr));

                return (tokens, Some('|'));
            },

            ('>', _) => panic!(),

            (x, ParserState::None) => {tempstr.push(x);ParserState::None}

            // i think this might be used for external links
            // ctr f "defines the operation of" on Cat and look in the ref
            // but idk

            // !!! /\ that was wrong
            (x, ParserState::OpenBracket) => {
                tempstr.push('[');
                tempstr.push(x);

                // dbg!(x);
                // will fail if [] is not only for https stuff


                ParserState::None
            },
            // same deal-io as above
            (x, ParserState::ClosedBracket) => {
                tempstr.push(']');
                tempstr.push(x);

                ParserState::None
            }

            x @ ((_, ParserState::OpenBrace) |
            (_, ParserState::ClosedBracket) |
            (_, ParserState::ClosedBrace)) => {
                dbg!(x);
                todo!()
            },
        };

    }

    (tokens, None)

}

fn parse_link(chars: &mut Chars) -> Link {

    let (mut return_tokens, Some(return_char)) = parse(chars) else {panic!("wikitext in article name")};

    assert_eq!(return_tokens.len(), 1);


    let Token::Text(article_name) = return_tokens.remove(0) else {panic!("wikitext in article name")};

    let display_name = if return_char == ']' {

        article_name.clone()

    } else if return_char == '|' {

        let (mut display_tokens, Some(']')) = parse(chars) else {panic!("wikitext in article name")};

        assert_eq!(display_tokens.len(), 1);

        let Token::Text(x) = display_tokens.remove(0) else {panic!("wikitext in article name")};

        x
    } else {panic!()};

    Link { article_name, display_name }

}

fn parse_template(chars: &mut Chars) -> Template {

    let (mut first_tokens, Some(mut char)) = parse(chars) else {panic!()};

    assert_eq!(first_tokens.len(), 1);

    let Token::Text(template_name) = first_tokens.remove(0) else {panic!()};

    let mut options = Vec::new();

    let mut option_num = 0;

    // while there still is more options to parse
    while char == '|' {

        // grab the parsed tokens
        let (mut options_tokens, Some(new_char)) = parse(chars) else { panic!()};

        // assign the new ending char
        char = new_char;

        // if the first param is text
        if let Token::Text(option) = &(options_tokens[0]) {

            // then either it is a named param
            if let Some(x) = option.find('=') {

                let re = Regex::new(r" *[a-zA-Z]* *= *").unwrap();

                assert!(re.is_match(option), "{}", option);

                // split the token in the name equal sign and rest of arg
                let Token::Text(mut option_name) = options_tokens.remove(0) else {unreachable!()};

                let mut eq = dbg!(option_name.split_off(x));

                let text_token = dbg!(eq.split_off(1));

                assert_eq!(eq, "=");

                options_tokens.insert(0, Token::Text(text_token));

                options.push((Either::Right(option_name), options_tokens));

            // or it just happens to be text at the start
            } else {
                dbg!("positional arg,", &options_tokens, option_num);

                options.push((Either::Left(option_num), options_tokens));

            }

        } else {
            dbg!("positional arg,", &options_tokens, option_num);

            options.push((Either::Left(option_num), options_tokens));
        }

        option_num += 1;

    }

    assert_eq!(char, '}');
    // assert_eq!('}', chars.next().unwrap()); // closing brace is consumed while returning

    Template {template_name, options}
}

fn parse_span(chars: &mut Chars, init_char: char) -> Span {

    // start grabbing the spans type
    let mut span_type = String::new();
    span_type.push(dbg!(init_char));
    let mut options = Vec::new();

    let self_closing;

    'span: loop {

        let x = dbg!(chars.next().unwrap());

        // if the spans name is over
        if x == ' ' {

            // start grabbing the options
            loop {

                let mut option_name = String::new();
                let mut option_val = String::new();

                let mut x = chars.next().unwrap();

                // clear the spaces
                while x == ' ' {
                    x = chars.next().unwrap();
                }

                // if its the last option deal with it
                if x == '>' {
                    self_closing = false;
                    break 'span
                } else if x == '/' {
                    assert_eq!('>', chars.next().unwrap());
                    self_closing = true;
                    break 'span;
                }
                dbg!(x);
                // grab the option name
                while x != ' ' && x != '=' {

                    dbg!(x);
                    option_name.push(x);
                    x = chars.next().unwrap();
                }
                dbg!(x);
                // dbg!(&option_name);
                // dbg!(&span_type);

                while x.is_whitespace() {x = chars.next().unwrap();}


                assert_eq!(x, '=', "{} {} {:?}", span_type, option_name, &options);

                // clear the equal sign
                x = chars.next().unwrap();

                while x.is_whitespace() {x = chars.next().unwrap();}

                if (x == '"') {

                    x = chars.next().unwrap();

                    while x != '"' {
                        option_val.push(x);
                        x = chars.next().unwrap();
                    }

                } else {

                    assert_eq!(span_type, "ref");

                    // consume till the un enclosed option is taken fully or we reach the end of tag
                    while !x.is_whitespace() {
                        option_val.push(x);
                        x = chars.next().unwrap();

                        if x == '>' {

                            options.push((option_name, option_val));

                            self_closing = false;
                            break 'span
                        } else if x == '/' {
                            todo!()
                        }
                    }
                }

                options.push((option_name, option_val));

            }

        } else if x == '>' {

            self_closing = false;
            break 'span;

        } else if x == '/' {

            assert_eq!('>', chars.next().unwrap());
            self_closing = true;
            break 'span;

        } else {

            assert!(x.is_alphabetic());
            span_type.push(x);

        }

    }

    let inner_text = if self_closing {

        Vec::new()

    } else {

        let (inner, Some('/')) = dbg!(parse(chars)) else {panic!()};

        let mut closing_span = String::new();

        let mut x = chars.next().unwrap();

        while x != '>' {
            closing_span.push(x);
            x = chars.next().unwrap()
        }

        assert_eq!(closing_span, span_type);

        inner
    };

    Span {
        span_type,
        options,
        inner_text,
    }

}

fn main() {

    // let mess = reqwest::blocking::get("https://api.wikimedia.org/core/v1/wikipedia/en/page/Cat_(Unix)").unwrap();
    //
    // let body: Value = mess.json().unwrap();
    //
    // let raw_page = body.as_object().unwrap().get("source").unwrap().as_str().unwrap();
    //
    // println!("{:?}", parse(&mut raw_page.chars()))


    // dbg!(parse(&mut "Hello World [[hi]] [[Hello Wolrd lame| Hi World]]".chars()));

    // dbg!(parse(&mut "{{For|other uses of cat|Cat (disambiguation)|Cat|}}".chars()));

//     dbg!(parse(&mut "{{Infobox software\n
// | name                   = cat\n
// | logo                   = \n
// | screenshot             = Cat-example-command.gif\n
// | screenshot size        = \n
// | caption                = \n
// | author                 = [[Ken Thompson]],<br/>[[Dennis Ritchie]]\n
// | developer              = [[AT&T Bell Laboratories]]\n
// | released               = {{Start date and age|1971|11|3}}\
// | latest release version = \n
// | latest release date    = \n
// | operating system       = [[Unix]], [[Unix-like]], [[Plan 9 from Bell Labs|Plan 9]],
//     [[Inferno (operating system)|Inferno]], [[ReactOS]]\n
// | platform               = [[Cross-platform]]\n
// | genre                  = [[Command (computing)|Command]]\n
// | license                = [[coreutils]]: [[GPLv3+]]<br/>ReactOS: [[GPLv2+]]\n
// | website                = \n
// }}".chars()));

    // dbg!(parse(&mut "{{For|other uses of cat|Cat (disambiguation)|Cat|}}".chars()));
}
