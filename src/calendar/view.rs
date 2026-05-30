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
}
