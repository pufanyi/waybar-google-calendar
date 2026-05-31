use crate::i18n::{month_name, translate};
use crate::storage::settings::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarViewMode {
    Days,
    Months,
    Years,
}

impl CalendarViewMode {
    pub fn next_level(self) -> Self {
        match self {
            Self::Days => Self::Months,
            Self::Months => Self::Years,
            Self::Years => Self::Days,
        }
    }
}

pub const YEAR_GRID_COUNT: i32 = 12;
pub const YEAR_PAGE_STEP: i32 = 10;

pub fn year_page_start(year: i32) -> i32 {
    year.div_euclid(YEAR_PAGE_STEP) * YEAR_PAGE_STEP
}

pub fn calendar_title_text(
    view: CalendarViewMode,
    year: i32,
    month: u32,
    lang: Language,
) -> String {
    match view {
        CalendarViewMode::Days => format!("{} {}", month_name(lang, month), year),
        CalendarViewMode::Months => year.to_string(),
        CalendarViewMode::Years => {
            let start = year_page_start(year);
            format!("{}-{}", start, start + YEAR_GRID_COUNT - 1)
        }
    }
}

pub fn calendar_title_icon_name(view: CalendarViewMode) -> &'static str {
    if view == CalendarViewMode::Years {
        "go-up-symbolic"
    } else {
        "go-down-symbolic"
    }
}

pub fn previous_calendar_tooltip(view: CalendarViewMode, lang: Language) -> &'static str {
    match view {
        CalendarViewMode::Days => translate(lang, "previous_month"),
        CalendarViewMode::Months => translate(lang, "previous_year"),
        CalendarViewMode::Years => translate(lang, "previous_years"),
    }
}

pub fn next_calendar_tooltip(view: CalendarViewMode, lang: Language) -> &'static str {
    match view {
        CalendarViewMode::Days => translate(lang, "next_month"),
        CalendarViewMode::Months => translate(lang, "next_year"),
        CalendarViewMode::Years => translate(lang, "next_years"),
    }
}

pub fn calendar_title_tooltip(view: CalendarViewMode, lang: Language) -> &'static str {
    match view {
        CalendarViewMode::Days => translate(lang, "choose_month"),
        CalendarViewMode::Months => translate(lang, "choose_year"),
        CalendarViewMode::Years => translate(lang, "return_to_days"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_view_cycles_through_picker_levels() {
        assert_eq!(
            CalendarViewMode::Days.next_level(),
            CalendarViewMode::Months
        );
        assert_eq!(
            CalendarViewMode::Months.next_level(),
            CalendarViewMode::Years
        );
        assert_eq!(CalendarViewMode::Years.next_level(), CalendarViewMode::Days);
    }

    #[test]
    fn year_page_starts_on_decade_boundary() {
        assert_eq!(year_page_start(2026), 2020);
        assert_eq!(year_page_start(2030), 2030);
    }

    #[test]
    fn calendar_title_text_matches_view_level() {
        assert_eq!(
            calendar_title_text(CalendarViewMode::Days, 2026, 5, Language::English),
            "May 2026"
        );
        assert_eq!(
            calendar_title_text(CalendarViewMode::Months, 2026, 5, Language::English),
            "2026"
        );
        assert_eq!(
            calendar_title_text(CalendarViewMode::Years, 2026, 5, Language::English),
            "2020-2031"
        );
    }
}
