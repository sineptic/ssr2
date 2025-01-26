pub enum TelegramInteraction {
    OneOf(Vec<String>),
    Text(String),
}

impl From<TelegramInteraction> for s_text_input_f::Block {
    fn from(interaction: TelegramInteraction) -> Self {
        match interaction {
            TelegramInteraction::OneOf(options) => s_text_input_f::Block::OneOf(options),
            TelegramInteraction::Text(content) => {
                s_text_input_f::Block::Paragraph(vec![s_text_input_f::ParagraphItem::Text(content)])
            }
        }
    }
}

impl TryFrom<s_text_input_f::Block> for TelegramInteraction {
    type Error = ();

    fn try_from(block: s_text_input_f::Block) -> Result<Self, Self::Error> {
        match block {
            s_text_input_f::Block::OneOf(options) => Ok(TelegramInteraction::OneOf(options)),
            s_text_input_f::Block::Paragraph(items) => match items.as_slice() {
                [s_text_input_f::ParagraphItem::Text(content)] => {
                    Ok(TelegramInteraction::Text(content.clone()))
                }
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}
