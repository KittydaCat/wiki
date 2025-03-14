// TODO
// Expand templates :(
// Space infront of lines make it weird
// Add wiki table support :(

use std::cell::RefCell;
use std::cmp::PartialEq;
use std::collections::HashSet;
use std::fmt::{Display, Formatter, Write};
use std::fs::{self, File};
use std::str::{Chars};
use serde_json::Value;
use either::Either;
use regex::Regex;
use colored;
use colored::Colorize;

thread_local!(static TAGS: RefCell<HashSet<String>> = RefCell::new(HashSet::<String>::new()));

#[derive(Debug,Clone)]
enum Token {
    Text(String),
    Link(Link),
    Template(Template),
    Span(Span),
}

impl Token {
    fn write(tokens: &[Token], f: &mut Formatter<'_>) -> std::fmt::Result {

        for x in tokens {

            write!(f, "{}", x)?;

        }
        Ok(())
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Text(x) => {
                f.write_str(x)
            }
            Token::Link(x) => {
                write!(f, "{}", x.display_name.blue())
            }
            Token::Template(x) => {write!(f, "{{{{ {} }}}}", &x.template_name)}
            Token::Span(x) => {
                write!(f, "<{:?}>", &x.span_type)?;
                Token::write(&x.inner_text, f)?;
                write!(f, "<{:?}>", &x.span_type)
            }
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
    span_type: SpanType,
    options: Vec<(String, String)>,
    inner_text: Vec<Token>
}

#[derive(Clone, Debug)]
enum SpanType {
    Code,
    KBD,
    Ref,
    Br,
    Other(String),
    Comment,
}

#[derive(Debug, Clone, PartialEq)]
enum ParserState {
    OpenBracket,
    ClosedBracket,
    OpenBrace,
    ClosedBrace,
    Apostrophe(u32),
    None,
}

#[derive(Debug, Clone, PartialEq)]
enum ParserGoal {
    DoubleBracket,
    // DoubleBrace,
    // PipeBrace,
    Span,
    None,
    // Pipe,
    EndTemplate,
}

fn parse(chars: &mut Chars, goal: ParserGoal) -> (Vec<Token>, Option<char>) {

    let mut state = ParserState::None;

    let mut tokens = Vec::new();

    let mut tempstr = String::new();

    while let Some(grabbed_char) = chars.next() {

        state = match (grabbed_char, state, &goal) {

            ('{', ParserState::None, _) => ParserState::OpenBrace,
            ('[', ParserState::None, _) => ParserState::OpenBracket,
            ('}', ParserState::None, ParserGoal::EndTemplate) => ParserState::ClosedBrace,
            (']', ParserState::None, ParserGoal::DoubleBracket) => ParserState::ClosedBracket,
            // ('\'', ParserState::None) => ParserState::Apostrophe(1),
            // ('\'', ParserState::Apostrophe(x)) => ParserState::Apostrophe(x+1),
            (_, ParserState::Apostrophe(_), _) => panic!(),

            ('[', ParserState::OpenBracket, _) => {
                tokens.push(Token::Text(tempstr));
                tempstr = String::new();

                tokens.push(Token::Link(parse_link(chars)));
                ParserState::None
            },

            ('{', ParserState::OpenBrace, _) => {
                tokens.push(Token::Text(tempstr));
                tempstr = String::new();

                tokens.push(Token::Template(parse_template(chars)));
                ParserState::None
            },

            ('|', ParserState::OpenBrace, _) => {
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

            (']', ParserState::ClosedBracket, x) => {

                assert_eq!(x, &ParserGoal::DoubleBracket);

                tokens.push(Token::Text(tempstr));

                return (tokens, Some(']'));

            }
            ('}', ParserState::ClosedBrace, x) => {

                assert_eq!(x, &ParserGoal::EndTemplate);

                tokens.push(Token::Text(tempstr));

                return (tokens, Some('}'));
            }

            ('<', ParserState::None, goal) => {
                let x = chars.next().unwrap();

                // if its a closing tag its not our problem
                if x == '/' {
                    tokens.push(Token::Text(tempstr));
                    tempstr = String::new();

                    assert_eq!(goal, &ParserGoal::Span);

                    return (tokens, Some('/'));
                } else if x.is_ascii_alphabetic() {
                    tokens.push(Token::Text(tempstr));
                    tempstr = String::new();

                    tokens.push(Token::Span(parse_span(chars, x)));
                } else {
                    assert_eq!(x, ' ');
                    tempstr.push('<');
                    tempstr.push(x);
                }
                ParserState::None
            },

            ('|', ref x, ParserGoal::DoubleBracket | ParserGoal::EndTemplate) => {

                assert_eq!(*x, ParserState::None);

                tokens.push(Token::Text(tempstr));

                return (tokens, Some('|'));
            },

            (x, ParserState::None, _) => {tempstr.push(x);ParserState::None}

            (x, ParserState::OpenBracket, _) => {



                tempstr.push('[');
                tempstr.push(x);

                ParserState::None
            },
            // same deal-io as above
            (x, ParserState::ClosedBracket, goal) => {

                assert_eq!(goal, &ParserGoal::DoubleBracket);

                tempstr.push(']');
                tempstr.push(x);

                ParserState::None
            }

            x @ ((_, ParserState::OpenBrace, _) | (_, ParserState::ClosedBrace, _)) => {
                dbg!(x);
                todo!()
            },
        };

    }

    (tokens, None)

}

fn parse_link(chars: &mut Chars) -> Link {

    // wikitext is allowed in display links
    let (mut return_tokens, Some(return_char)) = parse(chars, ParserGoal::DoubleBracket) else {panic!("wikitext in article name")};

    assert_eq!(return_tokens.len(), 1);

    let Token::Text(article_name) = return_tokens.remove(0) else {panic!("wikitext in article name")};

    let display_name = if return_char == ']' {

        article_name.clone()

    } else if return_char == '|' {

        let (mut display_tokens, Some(']')) = parse(chars, ParserGoal::DoubleBracket) else {panic!("wikitext in article name")};

        assert_eq!(display_tokens.len(), 1);

        let Token::Text(x) = display_tokens.remove(0) else {panic!("wikitext in article name")};

        x
    } else {panic!()};

    Link { article_name, display_name }

}

fn parse_template(chars: &mut Chars) -> Template {

    let (mut first_tokens, Some(mut char)) = parse(chars, ParserGoal::EndTemplate) else {panic!()};

    assert_eq!(first_tokens.len(), 1);

    let Token::Text(template_name) = first_tokens.remove(0) else {panic!()};

    let mut options = Vec::new();

    let mut option_num = 0;

    // while there still is more options to parse
    while char == '|' {

        // grab the parsed tokens
        let (mut options_tokens, Some(new_char)) = parse(chars, ParserGoal::EndTemplate) else { panic!()};

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
                dbg!("positional arg", &options_tokens, option_num, &template_name);

                options.push((Either::Left(option_num), options_tokens));

            }

        } else {
            dbg!("positional arg", &options_tokens, option_num, &template_name);

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
    let mut span_name = String::new();
    span_name.push(dbg!(init_char));
    let mut options = Vec::new();

    let self_closing;

    // we got a comment
    if init_char == '!' {

        let mut inner_str = String::new();

        assert_eq!(chars.next().unwrap(), '-');
        assert_eq!(chars.next().unwrap(), '-');

        let mut char1 = chars.next().unwrap();
        let mut char2 = chars.next().unwrap();
        let mut char3 = chars.next().unwrap();

        dbg!(char1, char2, char3);

        while !(char1 == '-' && char2 == '-' && char3 == '>') {

            dbg!(char1);

            inner_str.push(char1);
            char1 = char2;
            char2 = char3;
            char3 = chars.next().unwrap();
        }

        dbg!(char1, char2, char3, "return");

        return Span {
            span_type: SpanType::Comment,
            options,
            inner_text: vec![Token::Text(inner_str)],
        };
    }

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


                assert_eq!(x, '=', "{} {} {:?}", span_name, option_name, &options);

                // clear the equal sign
                x = chars.next().unwrap();

                while x.is_whitespace() {x = chars.next().unwrap();}

                if x == '"' {

                    x = chars.next().unwrap();

                    while x != '"' {
                        option_val.push(x);
                        x = chars.next().unwrap();
                    }

                } else {

                    assert_eq!(span_name, "ref");

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

            assert!(x.is_alphabetic(), "{}", &span_name);
            span_name.push(x);

        }

    }

    let inner_text = if self_closing {

        Vec::new()

    } else {

        let (inner, Some('/')) = dbg!(parse(chars, ParserGoal::Span)) else {panic!()};

        let mut closing_span = String::new();

        let mut x = chars.next().unwrap();

        while x != '>' {
            closing_span.push(x);
            x = chars.next().unwrap()
        }

        assert_eq!(closing_span, span_name);

        inner
    };

    let span_type = match span_name.as_str() {
        "ref" => SpanType::Ref,
        "code" => SpanType::Code,
        "kbd" => SpanType::KBD,
        "br" => SpanType::Br,
        _ => {
            TAGS.with_borrow_mut(|x| x.insert(span_name.clone()));
            SpanType::Other(span_name)
        }
    };

    Span {
        span_type,
        options,
        inner_text,
    }

}

fn main() {

    // let mess = reqwest::blocking::get("https://api.wikimedia.org/core/v1/wikipedia/en/page/Lisp_(programming_language)").unwrap();
    let mess = reqwest::blocking::get("https://api.wikimedia.org/core/v1/wikipedia/en/page/Cat_(Unix)").unwrap();

    let body: Value = mess.json().unwrap();

    let raw_page = body.as_object().unwrap().get("source").unwrap().as_str().unwrap();

    for x in parse(&mut raw_page.chars(), ParserGoal::None).0 {

        print!("{}", x);

    }
    
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
    dbg!(TAGS.with_borrow(|x| x.iter().map(String::clone).collect::<Vec<_>>()));
}
