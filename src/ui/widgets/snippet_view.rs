use std::collections::HashMap;

use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, text::Line, widgets::{Block, Borders, List, Padding}};

use crate::{document::Document, snippets::{Library, snippets::{Page, Snippet}}, ui::widgets::{self, document_view, metadata_view}};

pub struct Widget<'a> {
    snippet: &'a Snippet,
    library: &'a Library,
}

pub struct State {
    selected_page: SelectedPage,
}

struct Layout {
    document_area: Rect,
    metadata_area: Rect,
    links_area: Option<Rect>,
}

enum SelectedPage {
    Page(usize),
    Overview,
}

impl State {
    pub fn new() -> Self {
        State{
            selected_page: SelectedPage::Overview,
        }
    }

    pub fn select_page(&mut self, page_index: usize) {
        self.selected_page = SelectedPage::Page(page_index);
    }

    pub fn select_overview(&mut self) {
        self.selected_page = SelectedPage::Overview;
    }
}

impl<'a> Widget<'a> {
    pub fn new(snippet: &'a Snippet, library: &'a Library) -> Self {
        Widget{
            snippet,
            library,
        }
    }

    fn render_page_border(&self, area: Rect, buffer: &mut Buffer, selected_page_index: usize, selected_page: &Page) -> Rect {
        let bottom_title = {
            let one_based_index = selected_page_index + 1;
            let page_count = self.snippet.pages.len();

            let caption = match &selected_page.caption {
                Some(caption) => format!(" {}/{} {} ", one_based_index, page_count, caption),
                None => format!(" {}/{} ", one_based_index, page_count),
            };

            Line::raw(caption)
        };

        let block = Block::new().title_bottom(bottom_title).borders(Borders::ALL);
        let inner_area = block.inner(area);

        ratatui::widgets::Widget::render(block, area, buffer);

        inner_area
    }

    fn render_overview_border(&self, area: Rect, buffer: &mut Buffer) -> Rect {
        let bottom_title = Line::raw(" Snippet overview ");
        let block = Block::new().title_bottom(bottom_title).borders(Borders::ALL).padding(Padding::uniform(1));
        let inner_area = block.inner(area);

        ratatui::widgets::Widget::render(block, area, buffer);

        inner_area
    }

    fn render_document_viewer(&self, area: Rect, buffer: &mut Buffer, document: &Document) {
        let document_viewer = document_view::Widget::new(document);
        ratatui::widgets::Widget::render(document_viewer, area, buffer);
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

        ratatui::widgets::Widget::render(metadata_viewer, area, buffer);
    }

    fn render_links(&self, area: Rect, buffer: &mut Buffer) {
        let block = Block::new().borders(Borders::TOP).title_top(" See also ").title_alignment(ratatui::layout::HorizontalAlignment::Center);
        let block_inner_area = block.inner(area);
        let linked_nodes = self.snippet.links.iter().enumerate().map(|(index, linked_id)| {
            let snippet_description = self.library.snippet(*linked_id).description.as_str();

            format!("[{}] {}", index+1, snippet_description)
        }).collect::<Vec<_>>();
        let links_list = List::new(linked_nodes);

        ratatui::widgets::Widget::render(block, area, buffer);
        ratatui::widgets::Widget::render(links_list, block_inner_area, buffer);
    }

    fn render_overview(&self, area: Rect, buffer: &mut Buffer) {
        let inner_area = self.render_overview_border(area, buffer);
        let snippet = self.snippet;
        let overview = widgets::snippet_overview::Widget::new(&self.library, snippet);
        ratatui::widgets::Widget::render(overview, inner_area, buffer);
    }

    fn compute_layout(&self, area: Rect) -> Layout {
        let link_count = self.snippet.links.len();
        let [left_area, right_area] = ratatui::layout::Layout::horizontal([Constraint::Fill(1), Constraint::Length(20)]).areas(area);
        let metadata_area = right_area;

        if link_count > 0 {
            let [document_viewer_area, links_area] = ratatui::layout::Layout::vertical([Constraint::Fill(1), Constraint::Length((link_count + 1) as u16)]).areas(left_area);

            Layout {
                document_area: document_viewer_area,
                metadata_area,
                links_area: Some(links_area),
            }
        }
        else {
            let document_viewer_area = left_area;

            Layout {
                document_area: document_viewer_area,
                metadata_area,
                links_area: None,
            }
        }
    }

    fn render_page(&self, page_index: usize, area: Rect, buffer: &mut Buffer) {
        let selected_page = &self.snippet.pages[page_index];
        let inside_border_area = self.render_page_border(area, buffer, page_index, selected_page);

        let layout = self.compute_layout(inside_border_area);

        self.render_document_viewer(layout.document_area, buffer, selected_page.document());
        self.render_metadata_viewer(layout.metadata_area, buffer);
        if let Some(links_area) = layout.links_area {
            self.render_links(links_area, buffer);
        }
    }
}

impl<'a> ratatui::widgets::StatefulWidget for Widget<'a> {
    type State = State;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        match state.selected_page {
            SelectedPage::Overview => self.render_overview(area, buffer),
            SelectedPage::Page(page_index) => self.render_page(page_index, area, buffer),
        }
    }
}
