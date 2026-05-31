use crate::agenda::{AgendaApp, AgendaMsg};
use crate::calendar::date::{month_dates, today_for_timezone};
use crate::calendar::view::{
    CalendarViewMode, YEAR_GRID_COUNT, calendar_title_icon_name, calendar_title_text,
    calendar_title_tooltip, next_calendar_tooltip, previous_calendar_tooltip, year_page_start,
};
use crate::i18n::{month_name, translate, weekday_short};
use crate::ui::{classed_button, icon_button, label};
use adw::prelude::*;
use chrono::{Datelike, NaiveDate};
use relm4::ComponentSender;
use std::collections::BTreeSet;

pub(super) fn build(
    model: &AgendaApp,
    event_days: &BTreeSet<NaiveDate>,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let today = today_for_timezone(model.query.timezone.as_deref());
    let pane = gtk::Box::new(gtk::Orientation::Vertical, 12);
    pane.add_css_class("left-pane");
    pane.set_size_request(292, -1);
    pane.set_halign(gtk::Align::Start);
    pane.set_hexpand(false);

    pane.append(&header(model, sender.clone()));
    match model.calendar_view {
        CalendarViewMode::Days => {
            pane.append(&day_grid(model, event_days, today, sender.clone()));
        }
        CalendarViewMode::Months => {
            pane.append(&month_grid(model, today, sender.clone()));
        }
        CalendarViewMode::Years => {
            pane.append(&year_grid(model, today, sender.clone()));
        }
    }
    pane.append(&actions(model, today, sender));
    pane
}

fn header(model: &AgendaApp, sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let lang = model.language();
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let previous = icon_button(
        "go-previous-symbolic",
        &["nav-button", "icon-button"],
        previous_calendar_tooltip(model.calendar_view, lang),
    );
    let next = icon_button(
        "go-next-symbolic",
        &["nav-button", "icon-button"],
        next_calendar_tooltip(model.calendar_view, lang),
    );
    let title = title_button(model);
    title.set_hexpand(true);

    {
        let sender = sender.clone();
        previous.connect_clicked(move |_| sender.input(AgendaMsg::PreviousCalendarPage));
    }
    {
        let sender = sender.clone();
        title.connect_clicked(move |_| sender.input(AgendaMsg::CycleCalendarView));
    }
    {
        let sender = sender.clone();
        next.connect_clicked(move |_| sender.input(AgendaMsg::NextCalendarPage));
    }

    header.append(&previous);
    header.append(&title);
    header.append(&next);
    header
}

fn title_button(model: &AgendaApp) -> gtk::Button {
    let lang = model.language();
    let button = gtk::Button::new();
    button.set_cursor_from_name(Some("pointer"));
    button.add_css_class("calendar-title-button");
    button.set_tooltip_text(Some(calendar_title_tooltip(model.calendar_view, lang)));

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    content.add_css_class("calendar-title-content");
    content.set_halign(gtk::Align::Center);
    content.set_valign(gtk::Align::Center);
    content.append(&label(
        &calendar_title_text(
            model.calendar_view,
            model.calendar_year,
            model.calendar_month,
            lang,
        ),
        &["month-title"],
        0.5,
        false,
    ));

    let icon = gtk::Image::from_icon_name(calendar_title_icon_name(model.calendar_view));
    icon.add_css_class("calendar-title-icon");
    content.append(&icon);

    button.set_child(Some(&content));
    button
}

fn day_grid(
    model: &AgendaApp,
    event_days: &BTreeSet<NaiveDate>,
    today: NaiveDate,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Grid {
    let grid = gtk::Grid::builder()
        .column_spacing(5)
        .row_spacing(5)
        .build();
    grid.set_halign(gtk::Align::Center);

    for (col, weekday) in weekday_short(model.language()).iter().enumerate() {
        let item = label(weekday, &["weekday"], 0.5, false);
        item.set_size_request(30, 22);
        grid.attach(&item, col as i32, 0, 1, 1);
    }

    for (index, day) in month_dates(model.calendar_year, model.calendar_month)
        .iter()
        .enumerate()
    {
        let row = index / 7 + 1;
        let col = index % 7;
        let item = calendar_day_button(day.day(), event_days.contains(day));
        item.set_size_request(34, 34);
        style_day_button(&item, model, *day, today, event_days.contains(day));

        let selected_day = *day;
        let sender = sender.clone();
        item.connect_clicked(move |_| sender.input(AgendaMsg::SelectDay(selected_day)));

        grid.attach(&item, col as i32, row as i32, 1, 1);
    }

    grid
}

fn month_grid(
    model: &AgendaApp,
    today: NaiveDate,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Grid {
    let grid = picker_grid();
    for month in 1..=12 {
        let index = month - 1;
        let item = classed_button(
            month_name(model.language(), month),
            &["calendar-picker-cell"],
        );
        item.set_size_request(84, 40);
        if month == model.calendar_month {
            item.add_css_class("selected");
        }
        if today.year() == model.calendar_year && today.month() == month {
            item.add_css_class("today");
        }

        let sender = sender.clone();
        item.connect_clicked(move |_| sender.input(AgendaMsg::SelectMonth(month)));

        grid.attach(&item, (index % 3) as i32, (index / 3) as i32, 1, 1);
    }
    grid
}

fn year_grid(model: &AgendaApp, today: NaiveDate, sender: ComponentSender<AgendaApp>) -> gtk::Grid {
    let grid = picker_grid();
    let start = year_page_start(model.calendar_year);
    for offset in 0..YEAR_GRID_COUNT {
        let year = start + offset;
        let item = classed_button(&year.to_string(), &["calendar-picker-cell"]);
        item.set_size_request(84, 40);
        if year == model.calendar_year {
            item.add_css_class("selected");
        }
        if year == today.year() {
            item.add_css_class("today");
        }

        let sender = sender.clone();
        item.connect_clicked(move |_| sender.input(AgendaMsg::SelectYear(year)));

        grid.attach(&item, offset % 3, offset / 3, 1, 1);
    }
    grid
}

fn picker_grid() -> gtk::Grid {
    let grid = gtk::Grid::builder()
        .column_spacing(7)
        .row_spacing(7)
        .build();
    grid.add_css_class("calendar-picker-grid");
    grid.set_halign(gtk::Align::Center);
    grid
}

fn style_day_button(
    item: &gtk::Button,
    model: &AgendaApp,
    day: NaiveDate,
    today: NaiveDate,
    has_event: bool,
) {
    if day.month() != model.calendar_month {
        item.add_css_class("outside");
    }
    if day.weekday().number_from_monday() >= 6 {
        item.add_css_class("weekend");
    }
    if has_event {
        item.add_css_class("event-day");
    }
    if day == today {
        item.add_css_class("today");
    }
    if Some(day) == model.selected_day {
        item.add_css_class("selected");
    }
}

fn actions(model: &AgendaApp, today: NaiveDate, sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("calendar-actions");
    let all = classed_button(translate(model.language(), "all"), &["action-button"]);
    if model.selected_day.is_none() {
        all.add_css_class("selected");
    }
    let today_button = classed_button(translate(model.language(), "today"), &["action-button"]);
    if model.selected_day == Some(today) {
        today_button.add_css_class("selected");
    }

    {
        let sender = sender.clone();
        all.connect_clicked(move |_| sender.input(AgendaMsg::ClearSelection));
    }
    {
        let sender = sender.clone();
        today_button.connect_clicked(move |_| sender.input(AgendaMsg::Today));
    }

    actions.append(&all);
    actions.append(&today_button);
    actions
}

fn calendar_day_button(day: u32, has_event: bool) -> gtk::Button {
    let button = gtk::Button::new();
    button.set_cursor_from_name(Some("pointer"));
    button.add_css_class("date-cell");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.add_css_class("date-cell-content");
    content.set_halign(gtk::Align::Center);
    content.set_valign(gtk::Align::Center);

    let number = label(&day.to_string(), &["date-number"], 0.5, false);
    number.set_halign(gtk::Align::Center);
    content.append(&number);

    let dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    dot.add_css_class("event-dot");
    if !has_event {
        dot.add_css_class("empty");
    }
    dot.set_halign(gtk::Align::Center);
    content.append(&dot);

    button.set_child(Some(&content));
    button
}
