use crate::agenda::{AgendaApp, AgendaMsg};
use crate::calendar::date::{month_dates, month_name};
use crate::ui::{classed_button, label};
use adw::prelude::*;
use chrono::{Datelike, Local, NaiveDate};
use relm4::ComponentSender;
use std::collections::BTreeSet;

pub(super) fn build(
    model: &AgendaApp,
    event_days: &BTreeSet<NaiveDate>,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let today = Local::now().date_naive();
    let pane = gtk::Box::new(gtk::Orientation::Vertical, 12);
    pane.add_css_class("left-pane");
    pane.set_size_request(292, -1);
    pane.set_halign(gtk::Align::Start);
    pane.set_hexpand(false);

    pane.append(&header(model, sender.clone()));
    pane.append(&day_grid(model, event_days, today, sender.clone()));
    pane.append(&actions(model, today, sender));
    pane
}

fn header(model: &AgendaApp, sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let previous = classed_button("<", &["nav-button", "icon-button"]);
    let next = classed_button(">", &["nav-button", "icon-button"]);
    let title = label(
        &format!(
            "{} {}",
            month_name(model.calendar_month),
            model.calendar_year
        ),
        &["month-title"],
        0.5,
        false,
    );
    title.set_hexpand(true);

    {
        let sender = sender.clone();
        previous.connect_clicked(move |_| sender.input(AgendaMsg::PreviousMonth));
    }
    {
        let sender = sender.clone();
        next.connect_clicked(move |_| sender.input(AgendaMsg::NextMonth));
    }

    header.append(&previous);
    header.append(&title);
    header.append(&next);
    header
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

    for (col, weekday) in ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
        .iter()
        .enumerate()
    {
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
    let all = classed_button("All", &["action-button"]);
    if model.selected_day.is_none() {
        all.add_css_class("selected");
    }
    let today_button = classed_button("Today", &["action-button"]);
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
