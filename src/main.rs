// Todo:
// Add generic searching function
// Add display to Markup
// Make Markup not just og text
// - Add correctly displayed links to Double Bracket
// - Add support for different <> </> types

use std::fs::{self, File};
use serde_json::Value;
use colored::Colorize;
use either::Either;

type Text<'a> = Vec<Span<'a>>;

#[derive(Debug)]
struct Span<'a> {
    og_text: &'a str,
    span_type: SpanType<'a>,
}

#[derive(Debug)]
enum SpanType<'a> {
    Template(Template<'a>),
    ArticleLink(ArticleLink<'a>),
}

#[derive(Debug)]
struct Template<'a> {
    name: &'a str,
    options: Vec<(Either<&'a str, u32>, &'a str)>,
}

#[derive(Debug)]
struct ArticleLink<'a> {
    article_name: &'a str,
}

#[derive(Debug)]
struct Tag<'a> {
    name: &'a str,
    options: Vec<(Either<&'a str, u32>, &'a str)>,
    inner_text: &'a str,
}

#[derive(Clone, Copy)]
enum ParserState {
    Bracket,
    Brace,
    Angle,
    Apostrophe(u32),
    CBracket,
    CBrace,
    CAngle,
    None,
}

fn parse(text: &str) -> Vec<Span> {

    let mut chars = text.char_indices();

    let mut parser_state = ParserState::None;
    
    let mut spans = Vec::new();

    let mut inprogress: Vec<(SpanType, char)> = Vec::new();

    while let Some((mut pos, char)) = chars.next() {

        match (char, parser_state) {
            
            ('{', ParserState::None) => parser_state = ParserState::Brace,

            ('[', ParserState::None) => parser_state = ParserState::Bracket,

            ('\'', ParserState::None) => parser_state = ParserState::Apostrophe(0),

            ('\'', ParserState::Apostrophe(x)) => parser_state = ParserState::Apostrophe(x + 1),

            // ('<', _) =>

            ('{', ParserState::Brace) => {
                //start parsing a template

                // get the first arg
                let ending_pos;

                loop {

                    let (y, x) = chars.next().expect("TODO");

                    if (x == '|' || x =='}') {
                        ending_pos = y;
                        break;
                    }

                }

                let template_name = dbg!(&text[pos+1..ending_pos]);

                pos = ending_pos + 1;

                loop {
                    let ending_pos;

                    loop {

                        let (y, x) = chars.next().expect("TODO");

                        if (x == '|' || x =='}') {
                            ending_pos = y;
                            break;
                        }

                    }
                }

                todo!();

            }

            (_, ParserState::Brace | ParserState::Bracket) => parser_state = ParserState::None,        

            _ => todo!()

        }
    
    }

    spans

}

fn main() {

    // let mess = reqwest::blocking::get("https://api.wikimedia.org/core/v1/wikipedia/en/page/Cat_(Unix)").unwrap();

    // let body: Value = mess.json().unwrap();

    let raw_page = fs::read_to_string("rawtext.txt").unwrap();

    // let body: Value = serde_json::from_slice(&file).unwrap();
    //
    // let raw_page = body.as_object().unwrap().get("source").unwrap().as_str().unwrap();

    dbg!(parse(&raw_page));

    // dbg!(parse("Helle[[wor{ld [[ feak] as}da] asa]] dada"));
    // dbg!(parse("Helle{{wor[[ld [[ feak] asda] asa]] }}dada"));

    // dbg!(parse("Helle[[wor{ld [[ feak] as}da] asa]] dada Helle{{wor[[ld [[ feak] asda] asa]] }}dada"));

    // for x in parse(raw_page) {
    //     match x {
    //         Markup::DoubleBracket(txt) => {
    //             print!("{}", txt
    //                 .strip_prefix("[[").unwrap()
    //                 .strip_suffix("]]").unwrap()
    //                 .blue()
    //             )
    //         }
    //         Markup::DoubleBrace(_) => { print!("!") }
    //         Markup::Text(txt) => { print!("{}", txt) }
    //     }
    // }

    // println!("{}", page);
}
