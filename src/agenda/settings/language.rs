use crate::i18n::translate;
use crate::storage::settings::Language;
use adw::prelude::*;

pub(super) fn selected(combo: &gtk::ComboBoxText) -> Language {
    match combo.active_id().as_deref() {
        Some("chinese") => Language::Chinese,
        _ => Language::English,
    }
}

pub(super) fn set_options(combo: &gtk::ComboBoxText, lang: Language, selected: Language) {
    combo.remove_all();
    for language in Language::SETTINGS_ORDER {
        combo.append(
            Some(language_id(language)),
            translate(lang, language_id(language)),
        );
    }
    combo.set_active_id(Some(language_id(selected)));
}

pub(super) fn update_options(combo: &gtk::ComboBoxText, lang: Language) {
    let selected = selected(combo);
    set_options(combo, lang, selected);
}

pub(super) fn language_id(language: Language) -> &'static str {
    match language {
        Language::English => "english",
        Language::Chinese => "chinese",
    }
}
