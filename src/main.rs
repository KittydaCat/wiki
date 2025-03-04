// Todo:
// Add generic searching function
// Add display to Markup
// Make Markup not just og text
// - Add correctly displayed links to Double Bracket
// - Add support for different <> </> types

use serde_json::Value;
use colored::Colorize;

#[derive(Debug)]
enum Markup<'a> {
    DoubleBracket(&'a str),
    DoubleBrace(&'a str),
    Text(&'a str),
    Span(&'a str),
}

#[derive(Debug)]
struct ArticleLink<'a> {
    og_text: String,
    article_name: String,
    display_name: String,
}

enum ParserState {
    Bracket,
    Brace,
    None,
}

fn parse(mut text: &str) -> Vec<Markup> {

    let mut iter = text.char_indices();

    let mut tokens = Vec::new();

    let mut state = ParserState::None;

    while let Some((pos, char)) = iter.next() {

        // dbg!(pos,char);

        state = match (char, state) {

            ('[', ParserState::None) => {ParserState::Bracket},
            ('[', ParserState::Bracket) => {

                let (before, after) = dbg!(text.split_at(pos-1));

                tokens.push(Markup::Text(before));

                iter = after.char_indices();

                // to clear the two {{
                iter.next();
                iter.next();

                let mut indent = 2;

                let mut pos = 0;

                // TODO make this work with unmatched single paras
                while indent != 0 {

                    let (x, char) = iter.next()
                        .expect("Should not end with open brace");

                    pos = x;

                    match char {

                        '[' => {indent += 1;}
                        ']' => {indent -= 1;}

                        _ => {}
                    }

                }

                let (inside, remaining) = dbg!(after.split_at(pos+1));

                tokens.push(Markup::DoubleBracket(inside));

                text = remaining;

                iter = remaining.char_indices();

                ParserState::None

            },
            ('{', ParserState::None) => {ParserState::Brace},
            ('{', ParserState::Brace) => {

                let (before, after) = dbg!(text.split_at(pos-1));

                tokens.push(Markup::Text(before));

                iter = after.char_indices();

                // to clear the two {{
                iter.next();
                iter.next();

                let mut indent = 2;

                let mut pos = 0;

                // TODO make this work with unmatched single paras
                while indent != 0 {

                    let (x, char) = iter.next()
                        .expect("Should not end with open brace");

                    pos = x;

                    match char {

                        '{' => {indent += 1;}
                        '}' => {indent -= 1;}

                        _ => {}
                    }

                }

                let (inside, remaining) = dbg!(after.split_at(pos+1));

                tokens.push(Markup::DoubleBrace(inside));

                text = remaining;

                iter = remaining.char_indices();

                ParserState::None

            },

            (_, ParserState::Bracket | ParserState::Brace) => ParserState::None,
            (_, ParserState::None) => ParserState::None,
        }

    }

    tokens.push(Markup::Text(text));

    tokens

}

fn main() {

    let mess = reqwest::blocking::get("https://api.wikimedia.org/core/v1/wikipedia/en/page/Cat_(Unix)").unwrap();

    let body:Value = mess.json().unwrap();

    let raw_page = body.as_object().unwrap().get("source").unwrap().as_str().unwrap();

    // println!("{}", raw_page);

    // dbg!(parse("Helle[[wor{ld [[ feak] as}da] asa]] dada"));
    // dbg!(parse("Helle{{wor[[ld [[ feak] asda] asa]] }}dada"));

    // dbg!(parse("Helle[[wor{ld [[ feak] as}da] asa]] dada Helle{{wor[[ld [[ feak] asda] asa]] }}dada"));

    for x in parse(raw_page) {
        match x {
            Markup::DoubleBracket(txt) => {
                print!("{}", txt
                    .strip_prefix("[[").unwrap()
                    .strip_suffix("]]").unwrap()
                    .blue()
                )
            }
            Markup::DoubleBrace(_) => { print!("!") }
            Markup::Text(txt) => { print!("{}", txt) }
        }
    }

    // println!("{}", page);
}
