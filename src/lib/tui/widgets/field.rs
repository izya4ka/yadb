use ratatui::{
    style::{Style, Stylize},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};
use tui_input::Input;

use crate::lib::tui::widgets::path::PathHint;

#[derive(Debug, PartialEq)]
pub enum FieldType {
    Normal,
    Path,
}

#[derive(Debug, Default)]
pub struct FieldState {
    pub input: Input,
    pub is_selected: bool,
    pub is_editing: bool,
    pub is_only_numbers: bool,
}

impl FieldState {
    pub fn new(value: &str, is_selected: bool, is_only_numbers: bool) -> Self {
        Self {
            input: Input::new(value.to_string()),
            is_selected,
            is_editing: false,
            is_only_numbers,
        }
    }

    pub fn get(&self) -> &str {
        self.input.value()
    }
}

pub struct Field<'a> {
    title: &'a str,
    variant: FieldType,
}

impl StatefulWidget for Field<'_> {
    type State = FieldState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let scroll = state.input.visual_scroll(area.width as usize);
        let mut input = Paragraph::new(state.input.value())
            .block(
                Block::bordered()
                    .title(self.title)
                    .border_style(if state.is_editing {
                        Style::default().red()
                    } else if state.is_selected {
                        Style::default().blue()
                    } else {
                        Style::default()
                    }),
            )
            .scroll((0, scroll as u16));

        if state.is_editing {
            input = input.italic();
        }

        input.render(area, buf);

        if self.variant == FieldType::Path && state.is_editing {
            let mut box_area = area.clone();
            box_area.y = box_area.y + 2;
            PathHint::new(state.get()).render(box_area, buf);
        }
    }
}

impl<'a> Field<'a> {
    pub fn new(title: &'a str, variant: FieldType) -> Field<'a> {
        Self { title, variant }
    }
}
