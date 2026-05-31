use crate::storage::settings::{Language, WeekStart};

pub fn month_name(lang: Language, month: u32) -> &'static str {
    match lang {
        Language::Chinese => match month {
            1 => "一月",
            2 => "二月",
            3 => "三月",
            4 => "四月",
            5 => "五月",
            6 => "六月",
            7 => "七月",
            8 => "八月",
            9 => "九月",
            10 => "十月",
            11 => "十一月",
            12 => "十二月",
            _ => "日历",
        },
        Language::English => match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Calendar",
        },
    }
}

pub fn week_start_name(lang: Language, week_start: WeekStart) -> &'static str {
    match lang {
        Language::Chinese => match week_start {
            WeekStart::Monday => "星期一",
            WeekStart::Tuesday => "星期二",
            WeekStart::Wednesday => "星期三",
            WeekStart::Thursday => "星期四",
            WeekStart::Friday => "星期五",
            WeekStart::Saturday => "星期六",
            WeekStart::Sunday => "星期日",
        },
        Language::English => match week_start {
            WeekStart::Monday => "Monday",
            WeekStart::Tuesday => "Tuesday",
            WeekStart::Wednesday => "Wednesday",
            WeekStart::Thursday => "Thursday",
            WeekStart::Friday => "Friday",
            WeekStart::Saturday => "Saturday",
            WeekStart::Sunday => "Sunday",
        },
    }
}

pub fn weekday_short(lang: Language, week_start: WeekStart) -> [&'static str; 7] {
    let labels = match lang {
        Language::Chinese => ["一", "二", "三", "四", "五", "六", "日"],
        Language::English => ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"],
    };
    std::array::from_fn(|index| labels[(week_start.days_from_monday() as usize + index) % 7])
}
