use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Span}, widgets::{Block, Borders, Paragraph, StatefulWidget, Widget}};

use crate::{document::{self, Document, Style}, snippets::snippets::{Part, Snippet}, ui::widgets::document_view};

pub struct SnippetView<'a> {
    snippet: &'a Snippet,
}

pub struct SnippetViewState {
    selected_part: usize,
}

impl SnippetViewState {
    pub fn new() -> Self {
        SnippetViewState{
            selected_part: 0,
        }
    }

    pub fn select_first(&mut self) {
        self.selected_part = 0
    }

    pub fn select_next(&mut self) {
        self.selected_part += 1
    }

    pub fn select_previous(&mut self) {
        if self.selected_part >= 1 {
            self.selected_part -= 1
        }
    }

    fn ensure_within_bounds(&mut self, part_count: usize) {
        if self.selected_part >= part_count {
            self.selected_part = 0
        }
    }

    pub fn selected(&self) -> usize {
        self.selected_part
    }
}

impl<'a> SnippetView<'a> {
    pub fn new(snippet: &'a Snippet) -> Self {
        SnippetView{
            snippet,
        }
    }

    fn selected_snippet_part(&self, state: &mut SnippetViewState) -> (usize, &'a Part) {
        let snippet = self.snippet;

        state.ensure_within_bounds(snippet.parts.len());
        let selected_part_index = state.selected_part;

        (selected_part_index, &snippet.parts[selected_part_index])
    }
}

impl<'a> StatefulWidget for SnippetView<'a> {
    type State = SnippetViewState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let (selected_part_index, selected_part) = self.selected_snippet_part(state);

        let bottom_title = {
            let one_based_index = selected_part_index + 1;
            let part_count = self.snippet.parts.len();

            let caption = match selected_part.caption() {
                Some(caption) => format!(" {}/{} {} ", one_based_index, part_count, caption),
                None => format!(" {}/{} ", one_based_index, part_count),
            };

            Line::raw(caption)
        };
        let snippet_caption_block = Block::new().title_bottom(bottom_title).borders(Borders::ALL);
        let block_inner_area = snippet_caption_block.inner(area);

        let document_viewer = document_view::Widget::new(selected_part.document());

        snippet_caption_block.render(area, buffer);
        document_viewer.render(block_inner_area, buffer);
    }
}
