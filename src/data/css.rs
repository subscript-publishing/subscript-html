use core::fmt::Debug;
use std::collections::*;
use std::any::*;
use std::marker::*;
use std::cell::*;
use std::rc::*;
use serde::{Serialize, Deserialize};

///////////////////////////////////////////////////////////////////////////////
// STYLING - DATA
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Default, Serialize, Deserialize)]
pub struct Styling {
    pub(crate) default: StyleList,
    pub(crate) state: Vec<StateSelector>,
    pub(crate) animations: Vec<Animation>,
    pub(crate) media: Vec<MediaCondition>,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Default, Serialize, Deserialize)]
pub struct StyleList(pub(crate) Vec<Style>);

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct Style {
    pub(crate) property: String,
    pub(crate) value: String,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct Animation(pub(crate) Vec<AnimationInterval>);

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct AnimationInterval {
    pub(crate) value: String,
    pub(crate) style: StyleList,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct MediaCondition {
    pub(crate) condition: StyleList,
    pub(crate) body: StyleList,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct StateSelector {
    pub(crate) name: StateSelectorName,
    pub(crate) body: StyleList,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Serialize, Deserialize)]
pub(crate) enum StateSelectorName {
    Active,
    After,
    Before,
    Checked,
    Disabled,
    Empty,
    Enabled,
    FirstChild,
    FirstLetter,
    FirstLine,
    Focus,
    Hover,
    LastChild,
    OnlyChild,
    Link,
    Visited,
    SpellingError,
    GrammarError,
    Selection,
    Placeholder,
    Marker,
    Cue,
    Backdrop,
}


///////////////////////////////////////////////////////////////////////////////
// STYLING - API
///////////////////////////////////////////////////////////////////////////////

impl Styling {
    pub fn is_empty(&self) -> bool {
        self.default.0.is_empty() &&
        self.state.is_empty() &&
        self.animations.is_empty() &&
        self.media.is_empty()
    }
    pub fn extend(&mut self, new: Styling) {
        self.default.0.extend(new.default.0);
        self.state.extend(new.state);
        self.animations.extend(new.animations);
        self.media.extend(new.media);
    }
    pub fn add_style(&mut self, x: Style) {
        self.default.0.push(x);
    }
    pub fn add_state(&mut self, x: StateSelector) {
        self.state.push(x);
    }
    pub fn add_animation(&mut self, xs: Vec<AnimationInterval>) {
        self.animations.push(Animation(xs));
    }
    pub fn add_media(&mut self, condition: StyleList, body: StyleList) {
        self.media.push(MediaCondition{condition, body});
    }
}

impl StyleList {
    pub fn new() -> Self {
        StyleList(Vec::new())
    }
    pub fn push(&mut self, value: Style) {
        self.0.push(value);
    }
}

impl Style {
    pub fn new(property: &str, value: &str) -> Self {
        let property = String::from(property);
        let value = String::from(value);
        Style{property, value}
    }
}

impl StateSelector {
    pub fn new_active(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Active, body}
    }
    pub fn new_after(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::After, body}
    }
    pub fn new_before(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Before, body}
    }
    pub fn new_checked(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Checked, body}
    }
    pub fn new_disabled(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Disabled, body}
    }
    pub fn new_empty(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Empty, body}
    }
    pub fn new_enabled(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Enabled, body}
    }
    pub fn new_first_child(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::FirstChild, body}
    }
    pub fn new_first_letter(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::FirstLetter, body}
    }
    pub fn new_first_line(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::FirstLine, body}
    }
    pub fn new_focus(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Focus, body}
    }
    pub fn new_hover(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Hover, body}
    }
    pub fn new_last_child(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::LastChild, body}
    }
    pub fn new_only_child(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::OnlyChild, body}
    }
    pub fn new_link(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Link, body}
    }
    pub fn new_visited(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Visited, body}
    }
    pub fn new_spelling_error(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::SpellingError, body}
    }
    pub fn new_grammar_error(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::GrammarError, body}
    }
    pub fn new_selection(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Selection, body}
    }
    pub fn new_placeholder(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Placeholder, body}
    }
    pub fn new_marker(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Marker, body}
    }
    pub fn new_cue(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Cue, body}
    }
    pub fn new_backdrop(body: StyleList) -> Self {
        StateSelector{name: StateSelectorName::Backdrop, body}
    }
}

impl AnimationInterval {
    pub fn new(value: &str, style: StyleList) -> Self {
        AnimationInterval{value: String::from(value), style}
    }
}

impl StateSelectorName {
    pub fn as_str(&self) -> &str {
        match self {
            StateSelectorName::Active => ":active",
            StateSelectorName::After => "::after",
            StateSelectorName::Before => "::before",
            StateSelectorName::Checked => ":checked",
            StateSelectorName::Disabled => ":disabled",
            StateSelectorName::Empty => ":empty",
            StateSelectorName::Enabled => ":enabled",
            StateSelectorName::FirstChild => ":first-child",
            StateSelectorName::FirstLetter => "::first-letter",
            StateSelectorName::FirstLine => "::first-line",
            StateSelectorName::Focus => ":focus",
            StateSelectorName::Hover => ":hover",
            StateSelectorName::LastChild => ":last-child",
            StateSelectorName::OnlyChild => ":only-child",
            StateSelectorName::Link => ":link",
            StateSelectorName::Visited => ":visited",
            StateSelectorName::SpellingError => "::spelling-error",
            StateSelectorName::GrammarError => "::grammar-error",
            StateSelectorName::Selection => "::selection",
            StateSelectorName::Placeholder => "::placeholder",
            StateSelectorName::Marker => "::marker",
            StateSelectorName::Cue => "::cue",
            StateSelectorName::Backdrop => "::backdrop",
        }
    }
}



