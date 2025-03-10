use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum TelegramInteraction {
    OneOf(Vec<String>),
    Text(String),
    UserInput,
    Image(PathBuf),
}

impl From<TelegramInteraction> for s_text_input_f::Block {
    fn from(interaction: TelegramInteraction) -> Self {
        match interaction {
            TelegramInteraction::OneOf(options) => s_text_input_f::Block::OneOf(options),
            TelegramInteraction::Text(content) => {
                s_text_input_f::Block::Paragraph(vec![s_text_input_f::ParagraphItem::Text(content)])
            }
            TelegramInteraction::UserInput => {
                s_text_input_f::Block::Paragraph(vec![s_text_input_f::ParagraphItem::Placeholder])
            }
            TelegramInteraction::Image(path) => {
                s_text_input_f::Block::Paragraph(vec![s_text_input_f::ParagraphItem::Text(
                    format!("image: {}", path.display()),
                )])
            }
        }
    }
}

impl TryFrom<s_text_input_f::Block> for TelegramInteraction {
    type Error = &'static str;

    fn try_from(block: s_text_input_f::Block) -> Result<Self, Self::Error> {
        match block {
            s_text_input_f::Block::OneOf(options) => Ok(TelegramInteraction::OneOf(options)),
            s_text_input_f::Block::Paragraph(items) => match items.as_slice() {
                [s_text_input_f::ParagraphItem::Text(content)] => {
                    Ok(TelegramInteraction::Text(content.clone()))
                }
                [s_text_input_f::ParagraphItem::Placeholder] => Ok(TelegramInteraction::UserInput),
                _ => Err("Paragraph must contain exactly one item"),
            },
            _ => Err("Unsupported block type"),
        }
    }
}
