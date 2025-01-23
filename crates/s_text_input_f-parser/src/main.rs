use s_text_input_f_parser::parse_blocks;

fn test(input: &str) {
    let output = parse_blocks(input).unwrap();
    println!("------------------------------------------------------------------");
    println!("input: \n{input}");
    // println!("{output:#?}");
    println!();
}

fn main() {
    test(r#"
hello `world`!

`precipice` - a very steep side of a cliff or a mountain
precipice - a very `steep` side of a `cliff` or a `mountain`

`in advance` - before a particular time, or before doing a particular thing
in advance - `before` a particular time, or `before` doing a particular thing

`consequence` - a result of a particular action or situation, often one that is bad or not convenient
consequence - a `result` of a particular action or situation, often one that is `bad` or `not convenient`

- [ ] not correct
- [x] correct
- [ ] not correct
- [ ] not correct
- [x] correct
- [ ] not correct
- [ ] not correct
- [x] correct

- not correct
* correct
- not correct
- not correct

- not one_of
    "#.trim());
    // test("hello `world`!");

    // test("`precipice` - a very steep side of a cliff or a mountain");
    // test("precipice - a very `steep` side of a `cliff` or a `mountain`");

    // test("`in advance` - before a particular time, or before doing a particular thing");
    // test("in advance - `before` a particular time, or `before` doing a particular thing");

    // test("`consequence` - a result of a particular action or situation, often one that is bad or not convenient");
    // test("consequence - a `result` of a particular action or situation, often one that is `bad` or `not convenient`");

    //     test(
    //         r#"
    // - [ ] not correct
    // - [x] correct
    // - [ ] not correct
    // - [ ] not correct
    // - [x] correct
    // - [ ] not correct
    // - [ ] not correct
    // - [x] correct
    //     "#
    //         .trim(),
    //     );

    // test("- not one_of");
}
