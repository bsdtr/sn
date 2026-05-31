use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

pub fn render(content: &str) -> Text<'static> {
    let lines = render_lines(content);
    if lines.is_empty() {
        Text::from("")
    } else {
        Text::from(lines)
    }
}

pub fn line_count(content: &str) -> usize {
    if content.is_empty() {
        return 1;
    }
    render_lines(content).len().max(1)
}

fn render_lines(content: &str) -> Vec<Line<'static>> {
    let mut renderer = Renderer::new();
    let parser = Parser::new_ext(content, Options::all());
    for event in parser {
        renderer.handle(event);
    }
    renderer.finish()
}

struct Renderer {
    lines: Vec<Line<'static>>,
    current: Vec<Span<'static>>,
    styles: Vec<Style>,
    in_code_block: bool,
    list_depth: usize,
    ordered_counters: Vec<usize>,
    unordered_lists: Vec<bool>,
    link_url: Option<String>,
}

impl Renderer {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            current: Vec::new(),
            styles: vec![Style::default()],
            in_code_block: false,
            list_depth: 0,
            ordered_counters: Vec::new(),
            unordered_lists: Vec::new(),
            link_url: None,
        }
    }

    fn finish(mut self) -> Vec<Line<'static>> {
        self.flush_line();
        while self.lines.last().is_some_and(|line| line.spans.is_empty()) {
            self.lines.pop();
        }
        self.lines
    }

    fn handle(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag_end) => self.end_tag(tag_end),
            Event::Text(text) => self.push_text(text.to_string()),
            Event::Code(text) => self.push_span(
                text.to_string(),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Event::SoftBreak | Event::HardBreak => self.flush_line(),
            Event::Rule => {
                self.flush_line();
                self.lines
                    .push(Line::from(Span::styled("─".repeat(40), Style::default().fg(Color::DarkGray))));
                self.lines.push(Line::from(""));
            }
            Event::Html(text) | Event::InlineHtml(text) => {
                self.push_text(text.to_string());
            }
            _ => {}
        }
    }

    fn start_tag(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => {}
            Tag::Heading { level, .. } => {
                self.flush_line();
                self.styles.push(heading_style(level));
            }
            Tag::BlockQuote(_) => {
                self.flush_line();
                self.styles
                    .push(Style::default().fg(Color::Magenta).add_modifier(Modifier::ITALIC));
            }
            Tag::CodeBlock(_kind) => {
                self.flush_line();
                self.in_code_block = true;
            }
            Tag::List(start) => {
                self.list_depth += 1;
                if let Some(first) = start {
                    self.unordered_lists.push(false);
                    self.ordered_counters.push(first as usize);
                } else {
                    self.unordered_lists.push(true);
                    self.ordered_counters.push(0);
                }
            }
            Tag::Item => {
                self.flush_line();
                let indent = "  ".repeat(self.list_depth.saturating_sub(1));
                let prefix = if self.unordered_lists.last().copied().unwrap_or(true) {
                    format!("{indent}• ")
                } else {
                    let index = self.list_depth - 1;
                    let number = self.ordered_counters[index];
                    self.ordered_counters[index] = number + 1;
                    format!("{indent}{number}. ")
                };
                self.push_span(prefix, Style::default().fg(Color::Yellow));
            }
            Tag::Emphasis => self
                .styles
                .push(self.current_style().add_modifier(Modifier::ITALIC)),
            Tag::Strong => self
                .styles
                .push(self.current_style().add_modifier(Modifier::BOLD)),
            Tag::Strikethrough => self
                .styles
                .push(self.current_style().add_modifier(Modifier::CROSSED_OUT)),
            Tag::Link { dest_url, .. } => {
                self.link_url = Some(dest_url.to_string());
                self.styles
                    .push(Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED));
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, tag_end: TagEnd) {
        match tag_end {
            TagEnd::Paragraph => {
                self.flush_line();
                self.lines.push(Line::from(""));
            }
            TagEnd::Heading(_) => {
                self.styles.pop();
                self.flush_line();
                self.lines.push(Line::from(""));
            }
            TagEnd::BlockQuote(_) => {
                self.styles.pop();
                self.flush_line();
                self.lines.push(Line::from(""));
            }
            TagEnd::CodeBlock => {
                self.in_code_block = false;
                self.lines.push(Line::from(""));
            }
            TagEnd::List(_) => {
                if self.list_depth > 0 {
                    self.list_depth -= 1;
                }
                if !self.ordered_counters.is_empty() {
                    self.ordered_counters.pop();
                }
                if !self.unordered_lists.is_empty() {
                    self.unordered_lists.pop();
                }
                self.lines.push(Line::from(""));
            }
            TagEnd::Item => {
                self.flush_line();
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
                self.styles.pop();
            }
            TagEnd::Link => {
                if let Some(url) = self.link_url.take() {
                    self.push_span(format!(" ({url})"), Style::default().fg(Color::DarkGray));
                }
                self.styles.pop();
            }
            _ => {}
        }
    }

    fn push_text(&mut self, text: String) {
        if self.in_code_block {
            for line in text.lines() {
                self.lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Green),
                )));
            }
            return;
        }

        self.push_span(text, self.current_style());
    }

    fn push_span(&mut self, text: String, style: Style) {
        if text.is_empty() {
            return;
        }
        self.current.push(Span::styled(text, style));
    }

    fn flush_line(&mut self) {
        if self.current.is_empty() {
            return;
        }
        self.lines.push(Line::from(std::mem::take(&mut self.current)));
    }

    fn current_style(&self) -> Style {
        *self.styles.last().unwrap_or(&Style::default())
    }
}

fn heading_style(level: HeadingLevel) -> Style {
    match level {
        HeadingLevel::H1 => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        HeadingLevel::H2 => Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
        HeadingLevel::H3 => Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
        HeadingLevel::H4 => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        HeadingLevel::H5 | HeadingLevel::H6 => Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_heading_and_bold() {
        let lines = render_lines("# Title\n\n**bold** text");
        assert!(lines.len() >= 2);
        assert!(lines[0].spans[0].content.contains("Title"));
    }

    #[test]
    fn renders_list_items() {
        let lines = render_lines("- one\n- two");
        let rendered = lines
            .iter()
            .flat_map(|line| line.spans.iter().map(|span| span.content.as_ref()))
            .collect::<String>();
        assert!(rendered.contains('•'));
    }
}
