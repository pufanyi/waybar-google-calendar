use crate::date::{month_dates, month_name};
use crate::ui::{add_escape_to_close, classed_button, clear_grid, label};
use adw::prelude::*;
use chrono::{Datelike, Local, NaiveDate};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

#[derive(Debug)]
pub struct MonthApp {
    year: i32,
    month: u32,
    selected: NaiveDate,
}

#[derive(Debug)]
pub enum MonthMsg {
    Previous,
    Next,
    Today,
    Select(NaiveDate),
}

pub struct MonthWidgets {
    grid: gtk::Grid,
    title: gtk::Label,
    selected_label: gtk::Label,
}

impl SimpleComponent for MonthApp {
    type Init = ();
    type Input = MonthMsg;
    type Output = ();
    type Root = adw::ApplicationWindow;
    type Widgets = MonthWidgets;

    fn init_root() -> Self::Root {
        let root = adw::ApplicationWindow::builder()
            .title("Calendar")
            .default_width(360)
            .default_height(390)
            .resizable(false)
            .build();
        root.set_decorated(false);
        root
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let today = Local::now().date_naive();
        let model = MonthApp {
            year: today.year(),
            month: today.month(),
            selected: today,
        };

        let root_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root_box.add_css_class("panel");

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
        root_box.append(&topbar);

        let selected_label = label("", &["muted"], 0.5, false);
        root_box.append(&selected_label);

        let grid = gtk::Grid::builder()
            .column_spacing(6)
            .row_spacing(7)
            .build();
        root_box.append(&grid);

        let bottom = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let today_button = classed_button("Today", &["action-button"]);
        bottom.append(&today_button);
        root_box.append(&bottom);

        {
            let root = root.clone();
            close.connect_clicked(move |_| root.close());
        }
        {
            let sender = sender.clone();
            previous.connect_clicked(move |_| sender.input(MonthMsg::Previous));
        }
        {
            let sender = sender.clone();
            next.connect_clicked(move |_| sender.input(MonthMsg::Next));
        }
        {
            let sender = sender.clone();
            today_button.connect_clicked(move |_| sender.input(MonthMsg::Today));
        }

        root.set_content(Some(&root_box));
        add_escape_to_close(&root);

        let mut widgets = MonthWidgets {
            grid,
            title,
            selected_label,
        };
        render_month(&model, &mut widgets, sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            MonthMsg::Previous => {
                if self.month == 1 {
                    self.month = 12;
                    self.year -= 1;
                } else {
                    self.month -= 1;
                }
            }
            MonthMsg::Next => {
                if self.month == 12 {
                    self.month = 1;
                    self.year += 1;
                } else {
                    self.month += 1;
                }
            }
            MonthMsg::Today => {
                let today = Local::now().date_naive();
                self.year = today.year();
                self.month = today.month();
                self.selected = today;
            }
            MonthMsg::Select(day) => {
                self.selected = day;
                self.year = day.year();
                self.month = day.month();
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        render_month(self, widgets, sender);
    }
}

fn render_month(model: &MonthApp, widgets: &mut MonthWidgets, sender: ComponentSender<MonthApp>) {
    clear_grid(&widgets.grid);

    widgets
        .title
        .set_text(&format!("{} {}", month_name(model.month), model.year));
    widgets
        .selected_label
        .set_text(&model.selected.format("%A, %B %-d, %Y").to_string());

    for (col, weekday) in ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
        .iter()
        .enumerate()
    {
        let item = label(weekday, &["weekday"], 0.5, false);
        item.set_size_request(40, 22);
        widgets.grid.attach(&item, col as i32, 0, 1, 1);
    }

    let today = Local::now().date_naive();
    for (index, day) in month_dates(model.year, model.month).iter().enumerate() {
        let row = index / 7 + 1;
        let col = index % 7;
        let item = classed_button(&day.day().to_string(), &["day"]);
        item.set_size_request(40, 36);
        if day.month() != model.month {
            item.add_css_class("outside");
        }
        if day.weekday().number_from_monday() >= 6 {
            item.add_css_class("weekend");
        }
        if *day == today {
            item.add_css_class("today");
        }
        if *day == model.selected {
            item.add_css_class("selected");
        }

        let selected_day = *day;
        let sender = sender.clone();
        item.connect_clicked(move |_| sender.input(MonthMsg::Select(selected_day)));

        widgets.grid.attach(&item, col as i32, row as i32, 1, 1);
    }
}
