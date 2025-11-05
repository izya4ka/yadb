use ratatui::{
    layout::{self, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};
use tui_input::Input;

use crate::lib::tui::widgets::path_hint::{PathHint, PathHintState};

#[derive(Debug, Default, PartialEq)]
pub enum FieldType {
    #[default]
    Normal,
    Path(PathHintState),
}

#[derive(Debug, Default)]
pub struct FieldState {
    pub input: Input,
    pub is_selected: bool,
    pub is_editing: bool,
    pub is_only_numbers: bool,
    pub field_type: FieldType,
}

impl FieldState {
    pub fn new(
        value: &str,
        is_selected: bool,
        is_only_numbers: bool,
        field_type: FieldType,
    ) -> Self {
        Self {
            input: Input::new(value.to_string()),
            is_selected,
            is_editing: false,
            is_only_numbers,
            field_type,
        }
    }

    pub fn get(&self) -> &str {
        self.input.value()
    }
}

pub struct Field<'a> {
    title: &'a str,
}

impl StatefulWidget for Field<'_> {
    type State = FieldState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let layout: [Rect; 1] =
            Layout::new(layout::Direction::Vertical, [Constraint::Length(3)]).areas(area);

        let scroll = state.input.visual_scroll(layout[0].width as usize);
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

        input.render(layout[0], buf);

        if let FieldType::Path(path_hint) = &mut state.field_type
            && state.is_editing
        {
            let mut box_area = area;
            box_area.y += 2;
            box_area.x += 1;
            PathHint::new().render(box_area, buf, path_hint);
        }
    }
}

impl<'a> Field<'a> {
    pub fn new(title: &'a str) -> Field<'a> {
        Self { title }
    }
}
