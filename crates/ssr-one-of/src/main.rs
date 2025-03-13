type Task = ssr_algorithms::fsrs::Task;
type Facade<'a> = ssr_facade::stateful::Facade<'a, Task>;
const PATH: &str = "storage.json";

use ssr_core::tasks_facade::TasksFacade;

fn main() {
    let file = std::fs::read_to_string(PATH).unwrap();
    let mut facade: Facade = serde_json::from_str(&file).unwrap();
    let items = [
        "arms",      // 0
        "eyebrows",  // 1
        "hair",      // 2
        "hand",      // 3
        "hands",     // 4
        "head",      // 5
        "nails",     // 6
        "nose",      // 7
        "shoulders", // 8
        "teeth",     // 9
        "thumb",     // 10
        "toes",      // 11
    ];
    let options = s_text_input_f::Block::one_of(items);

    let tasks = [
        ("bite your _", vec![6]),
        ("blow your _", vec![7]),
        ("brush your _", vec![9, 2]),
        ("comb your _", vec![2]),
        ("fold your _", vec![0]),
        ("hold somebody's _", vec![3]),
        ("touch your _", vec![11]),
        ("suck your _", vec![10]),
        ("shake _", vec![4]),
        ("shrug your _", vec![8]),
        ("shake your _", vec![5]),
        ("raise your _", vec![1]),
    ];

    for (input, mut answers) in tasks {
        let blocks = vec![
            s_text_input_f::Block::Paragraph(vec![s_text_input_f::ParagraphItem::Text(
                input.into(),
            )]),
            options.clone(),
        ];
        let first_answer = vec![vec![], vec![answers.pop().unwrap().to_string()]];
        let other_answers = answers
            .into_iter()
            .map(|a| vec![vec![], vec![a.to_string()]])
            .collect::<Vec<_>>();
        let task = Task::new(blocks, first_answer, other_answers);
        facade.insert(task);
    }

    std::fs::write(PATH, serde_json::to_string_pretty(&facade).unwrap()).unwrap();
}
