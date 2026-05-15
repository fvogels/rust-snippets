use std::collections::HashMap;

use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Rect}, text::Line, widgets::{Block, Borders, List, StatefulWidget, Widget}};

use crate::{document::Document, snippets::{Library, snippets::{Part, Snippet}}, ui::widgets::{document_view, metadata_view::{self, Category}}};

pub struct SnippetView<'a> {
    snippet: &'a Snippet,
    library: &'a Library,
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
    pub fn new(snippet: &'a Snippet, library: &'a Library) -> Self {
        SnippetView{
            snippet,
            library,
        }
    }

    fn selected_snippet_part(&self, state: &mut SnippetViewState) -> (usize, &'a Part) {
        let snippet = self.snippet;

        state.ensure_within_bounds(snippet.parts.len());
        let selected_part_index = state.selected_part;

        (selected_part_index, &snippet.parts[selected_part_index])
    }

    fn render_border(&self, area: Rect, buffer: &mut Buffer, selected_part_index: usize, selected_part: &Part) -> Rect {
        let bottom_title = {
            let one_based_index = selected_part_index + 1;
            let part_count = self.snippet.parts.len();

            let caption = match &selected_part.caption {
                Some(caption) => format!(" {}/{} {} ", one_based_index, part_count, caption),
                None => format!(" {}/{} ", one_based_index, part_count),
            };

            Line::raw(caption)
        };

        let block = Block::new().title_bottom(bottom_title).borders(Borders::ALL);
        let inner_area = block.inner(area);

        block.render(area, buffer);

        inner_area
    }

    fn render_document_viewer(&self, area: Rect, buffer: &mut Buffer, document: &Document) {
        let document_viewer = document_view::Widget::new(document);
        document_viewer.render(area, buffer);
    }

    fn render_metadata_viewer(&self, area: Rect, buffer: &mut Buffer) {
        let metadata_viewer = {
            let mut category_table = HashMap::new();

            for tag in &self.snippet.tags {
                let category = category_table.entry(tag.category.clone()).or_insert_with(|| metadata_view::Category { caption: tag.category.clone(), entries: Vec::new()  });
                category.entries.push(tag.name.clone());
            }

            let mut categories = category_table.into_values().collect::<Vec<_>>();
            categories.sort_by(|c1, c2| c1.caption.cmp(&c2.caption));

            for category in &mut categories {
                category.entries.sort();
            }

            metadata_view::Widget::new(categories)
        };
        metadata_viewer.render(area, buffer);
    }

    fn render_links(&self, area: Rect, buffer: &mut Buffer) {
        let block = Block::new().borders(Borders::TOP).title("See also");
        let block_inner_area = block.inner(area);
        let linked_nodes = self.snippet.links.iter().map(|linked_id|
            self.library.snippet(*linked_id).description.as_str()
        ).collect::<Vec<_>>();
        let links_list = List::new(linked_nodes);

        block.render(area, buffer);
        ratatui::widgets::Widget::render(links_list, block_inner_area, buffer);
    }
}

impl<'a> StatefulWidget for SnippetView<'a> {
    type State = SnippetViewState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let (selected_part_index, selected_part) = self.selected_snippet_part(state);
        let link_count = self.snippet.links.len();

        let area = self.render_border(area, buffer, selected_part_index, selected_part);

        // Compute layout
        let (document_viewer_area, metadata_area, links_area) = {
            let [left_area, right_area] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(20)]).areas(area);
            let metadata_area = right_area;

            if link_count > 0 {
                let [document_viewer_area, links_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length((link_count + 1) as u16)]).areas(left_area);
                (document_viewer_area, metadata_area, Some(links_area))
            }
            else {
                let document_viewer_area = left_area;
                (document_viewer_area, metadata_area, None)
            }
        };

        self.render_document_viewer(document_viewer_area, buffer, selected_part.document());
        self.render_metadata_viewer(metadata_area, buffer);

        if let Some(links_area) = links_area {
            self.render_links(links_area, buffer);
        }
    }
}
