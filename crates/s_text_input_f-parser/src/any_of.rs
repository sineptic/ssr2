use chumsky::prelude::*;

pub struct CorrectAnyOf {
    pub variants: Vec<String>,
    pub correct: Vec<usize>,
}

pub(crate) fn any_of_parser() -> impl Parser<char, CorrectAnyOf, Error = Simple<char>> {
    let eol = || choice((just('\n').ignored(), end().ignored()));
    let variant = || {
        take_until(eol())
            .map(|(content, _)| content.into_iter().collect::<String>().trim().to_owned())
    };
    let correct_variant = || just("- [x] ").ignore_then(variant()).map(|x| (true, x));
    let wrong_variant = || just("- [ ] ").ignore_then(variant()).map(|x| (false, x));
    let variant = || choice((correct_variant(), wrong_variant()));

    variant().repeated().at_least(1).map(|lines| {
        let correct = lines
            .iter()
            .enumerate()
            .filter(|(_, (is_correct, _))| *is_correct)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        let variants = lines
            .into_iter()
            .map(|(_, content)| content)
            .collect::<Vec<_>>();
        CorrectAnyOf { variants, correct }
    })
}
