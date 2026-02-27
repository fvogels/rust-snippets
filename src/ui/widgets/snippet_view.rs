use ratatui::{buffer::Buffer, layout::Rect, text::Line, widgets::{Block, Borders, StatefulWidget, Widget}};

use crate::{snippets::snippets::{Part, Snippet}, ui::syntax::SyntaxHighlighter};

pub struct SnippetView<'a> {
    snippet: &'a Snippet,
    syntax_highlighter: &'a SyntaxHighlighter,
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
}

impl<'a> SnippetView<'a> {
    pub fn new(snippet: &'a Snippet, syntax_highlighter: &'a SyntaxHighlighter) -> Self {
        SnippetView{
            snippet,
            syntax_highlighter,
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
        let one_based_index = selected_part_index + 1;
        let part_count = self.snippet.parts.len();
        let snippet_caption = match selected_part.attributes.get("caption") {
            Some(caption) => format!(" {}/{} {} ", one_based_index, part_count, caption),
            None => format!(" {}/{} ", one_based_index, part_count),
        };
        let lines = selected_part.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
        let bottom_title = Line::raw(snippet_caption);
        let snippet_caption_block = Block::new().title_bottom(bottom_title).borders(Borders::ALL);
        let paragraph = self.syntax_highlighter.highlight_lines("Go", lines.into_iter()).unwrap().block(snippet_caption_block);
        paragraph.render(area, buffer)
    }
}