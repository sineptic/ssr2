use chumsky::prelude::*;

pub struct CorrectOneOf {
    pub variants: Vec<String>,
    pub correct: usize,
}

pub(crate) fn one_of_parser() -> impl Parser<char, CorrectOneOf, Error = Simple<char>> {
    let eol = || choice((just('\n').ignored(), end().ignored()));
    let variant = || {
        take_until(eol())
            .map(|(content, _)| content.into_iter().collect::<String>().trim().to_owned())
    };
    let correct_variant = || just("* ").ignore_then(variant());
    let wrong_variant = || just("- ").ignore_then(variant());
    wrong_variant()
        .repeated()
        .then(correct_variant())
        .then(wrong_variant().repeated())
        .map(|((wrong1, correct), wrong2)| {
            let correct_answer = wrong1.len();

            CorrectOneOf {
                correct: correct_answer,
                variants: [wrong1, vec![correct], wrong2].concat(),
            }
        })
}
