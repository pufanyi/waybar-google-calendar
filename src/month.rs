use crate::date::{month_dates, month_name};
use crate::ui::{add_escape_to_close, classed_button, clear_grid, label};
use adw::prelude::*;
use chrono::{Datelike, Local, NaiveDate};
use std::cell::RefCell;
use std::rc::Rc;

pub fn build_window(app: &adw::Application) {
    let today = Local::now().date_naive();
    let state = Rc::new(RefCell::new(MonthState {
        year: today.year(),
        month: today.month(),
        selected: today,
    }));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Calendar")
        .default_width(360)
        .default_height(390)
        .resizable(false)
        .build();
    window.set_decorated(false);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
    root.add_css_class("panel");

    let topbar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    topbar.add_css_class("topbar");

    let previous = classed_button("<", &["nav-button"]);
    let next = classed_button(">", &["nav-button"]);
    let close = classed_button("x", &["close-button"]);
    let title = label("", &["title"], 0.5, false);
    title.set_hexpand(true);

    topbar.append(&previous);
    topbar.append(&title);
    topbar.append(&next);
    topbar.append(&close);
    root.append(&topbar);

    let selected_label = label("", &["muted"], 0.5, false);
    root.append(&selected_label);

    let grid = gtk::Grid::builder()
        .column_spacing(6)
        .row_spacing(7)
        .build();
    root.append(&grid);

    let bottom = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let today_button = classed_button("Today", &["action-button"]);
    bottom.append(&today_button);
    root.append(&bottom);

    {
        let window = window.clone();
        close.connect_clicked(move |_| window.close());
    }

    {
        let state = state.clone();
        let grid = grid.clone();
        let title = title.clone();
        let selected_label = selected_label.clone();
        previous.connect_clicked(move |_| {
            {
                let mut state = state.borrow_mut();
                if state.month == 1 {
                    state.month = 12;
                    state.year -= 1;
                } else {
                    state.month -= 1;
                }
            }
            render_month(&grid, &title, &selected_label, state.clone());
        });
    }

    {
        let state = state.clone();
        let grid = grid.clone();
        let title = title.clone();
        let selected_label = selected_label.clone();
        next.connect_clicked(move |_| {
            {
                let mut state = state.borrow_mut();
                if state.month == 12 {
                    state.month = 1;
                    state.year += 1;
                } else {
                    state.month += 1;
                }
            }
            render_month(&grid, &title, &selected_label, state.clone());
        });
    }

    {
        let state = state.clone();
        let grid = grid.clone();
        let title = title.clone();
        let selected_label = selected_label.clone();
        today_button.connect_clicked(move |_| {
            {
                let mut state = state.borrow_mut();
                state.year = today.year();
                state.month = today.month();
                state.selected = today;
            }
            render_month(&grid, &title, &selected_label, state.clone());
        });
    }

    window.set_content(Some(&root));
    add_escape_to_close(&window);
    render_month(&grid, &title, &selected_label, state);
    window.present();
}

#[derive(Debug, Clone)]
struct MonthState {
    year: i32,
    month: u32,
    selected: NaiveDate,
}

fn render_month(
    grid: &gtk::Grid,
    title: &gtk::Label,
    selected_label: &gtk::Label,
    state: Rc<RefCell<MonthState>>,
) {
    clear_grid(grid);

    let state_snapshot = state.borrow().clone();
    title.set_text(&format!(
        "{} {}",
        month_name(state_snapshot.month),
        state_snapshot.year
    ));
    selected_label.set_text(&state_snapshot.selected.format("%A, %B %-d, %Y").to_string());

    for (col, weekday) in ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
        .iter()
        .enumerate()
    {
        let item = label(weekday, &["weekday"], 0.5, false);
        item.set_size_request(40, 22);
        grid.attach(&item, col as i32, 0, 1, 1);
    }

    let today = Local::now().date_naive();
    for (index, day) in month_dates(state_snapshot.year, state_snapshot.month)
        .iter()
        .enumerate()
    {
        let row = index / 7 + 1;
        let col = index % 7;
        let item = classed_button(&day.day().to_string(), &["day"]);
        item.set_size_request(40, 36);
        if day.month() != state_snapshot.month {
            item.add_css_class("outside");
        }
        if day.weekday().number_from_monday() >= 6 {
            item.add_css_class("weekend");
        }
        if *day == today {
            item.add_css_class("today");
        }
        if *day == state_snapshot.selected {
            item.add_css_class("selected");
        }

        {
            let state = state.clone();
            let grid = grid.clone();
            let title = title.clone();
            let selected_label = selected_label.clone();
            let selected_day = *day;
            item.connect_clicked(move |_| {
                {
                    let mut state = state.borrow_mut();
                    state.selected = selected_day;
                    state.year = selected_day.year();
                    state.month = selected_day.month();
                }
                render_month(&grid, &title, &selected_label, state.clone());
            });
        }

        grid.attach(&item, col as i32, row as i32, 1, 1);
    }
}
