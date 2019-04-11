use std::collections::HashMap;
use std::io::{BufReader, Cursor as IoCursor};

use syntect::highlighting::{Highlighter, Style as SyntectStyle, Theme, ThemeSet};
use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};
use xi_rope::spans::{Spans, SpansBuilder};
use xi_rope::{Cursor, LinesMetric, Rope};

use crate::style::{Style, StyleId};

//const DEFAULT_THEME: &str = "../assets/InspiredGitHub.tmTheme";

pub(crate) struct HighlightState {
    syntax_set: SyntaxSet,
    theme: Theme,
    state: Internal,
}

#[derive(Debug, Default, Clone)]
struct Internal {
    parse_state: Vec<(ParseState, ScopeStack)>,
    style_table: HashMap<Style, StyleId>,
    new_styles: Option<Vec<(StyleId, Style)>>,
    next_style_id: StyleId,
}

impl HighlightState {
    pub(crate) fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_nonewlines();
        let theme_data = include_str!("../assets/InspiredGitHub.tmTheme");
        let mut reader = BufReader::new(IoCursor::new(theme_data));
        let theme = ThemeSet::load_from_reader(&mut reader).expect("failed to load default theme");

        HighlightState { syntax_set, theme, state: Internal::default() }
    }

    pub(crate) fn highlight_all(&mut self, text: &Rope) -> Spans<StyleId> {
        let HighlightState { syntax_set, theme, state } = self;
        state.highlight_all(text, syntax_set, &theme)
    }

    /// Returns any newly defined styles. These should be sent to the client.
    pub(crate) fn take_new_styles(&mut self) -> Option<Vec<(StyleId, Style)>> {
        self.state.new_styles.take()
    }
}

impl Internal {
    pub(crate) fn highlight_all(
        &mut self,
        text: &Rope,
        syntax_set: &SyntaxSet,
        theme: &Theme,
    ) -> Spans<StyleId> {
        self.parse_state.clear();

        let syntax = syntax_set.find_syntax_by_name("Rust").expect("no syntax 'rust' found");
        let highlighter = Highlighter::new(theme);

        let mut b = SpansBuilder::new(text.len());
        let mut parse_state = ParseState::new(syntax);
        let mut scope_state = ScopeStack::new();
        let mut cursor = Cursor::new(text, 0);
        let mut total_offset = 0;

        while let Some(next_line) = cursor.next::<LinesMetric>() {
            let line = text.slice_to_cow(total_offset..next_line);
            let mut last_pos = 0;
            let ops = parse_state.parse_line(line.trim_right_matches('\n'), syntax_set);
            for (pos, batch) in ops {
                if !scope_state.is_empty() {
                    let start = total_offset + last_pos;
                    let end = start + (pos - last_pos);
                    if start != end {
                        let style = highlighter.style_for_stack(scope_state.as_slice());
                        let id = self.id_for_style(style);
                        b.add_span(start..end, id);
                    }
                }
                last_pos = pos;
                scope_state.apply(&batch);
            }
            // add EOL span:
            let start = total_offset + last_pos;
            let end = start + (line.len() - last_pos);
            let style = highlighter.style_for_stack(scope_state.as_slice());
            let id = self.id_for_style(style);
            b.add_span(start..end, id);
            total_offset += line.len();
        }
        b.build()
    }

    fn id_for_style(&mut self, style: SyntectStyle) -> StyleId {
        use std::collections::hash_map::Entry;

        let style = Style::from(style);
        match self.style_table.entry(style) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let value = self.next_style_id;
                self.next_style_id += 1;
                entry.insert(value);
                self.new_styles.get_or_insert(Vec::new()).push((value, style));
                value
            }
        }
    }
}
