use crate::i18n::translate;
use crate::storage::settings::Language;
use crate::ui::set_drop_down_strings;

pub(super) fn selected(dropdown: &gtk::DropDown) -> Language {
    Language::SETTINGS_ORDER
        .get(dropdown.selected() as usize)
        .copied()
        .unwrap_or_default()
}

pub(super) fn set_options(dropdown: &gtk::DropDown, lang: Language, selected: Language) {
    let labels = Language::SETTINGS_ORDER.map(|language| translate(lang, language_id(language)));
    set_drop_down_strings(dropdown, &labels, language_index(selected));
}

pub(super) fn update_options(dropdown: &gtk::DropDown, lang: Language) {
    let selected = selected(dropdown);
    set_options(dropdown, lang, selected);
}

pub(super) fn language_index(language: Language) -> usize {
    Language::SETTINGS_ORDER
        .iter()
        .position(|candidate| *candidate == language)
        .unwrap_or_default()
}

fn language_id(language: Language) -> &'static str {
    match language {
        Language::English => "english",
        Language::Chinese => "chinese",
    }
}
