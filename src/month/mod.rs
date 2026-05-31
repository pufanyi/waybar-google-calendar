use crate::calendar::date::{month_dates, month_name, shift_month};
use crate::calendar::view::{CalendarViewMode, YEAR_GRID_COUNT, YEAR_PAGE_STEP, year_page_start};
use crate::ui::{add_escape_action, classed_button, clear_grid, icon_button, label};
use adw::prelude::*;
use chrono::{Datelike, Local, NaiveDate};
use relm4::{Component, ComponentParts, ComponentSender};

#[derive(Debug)]
pub struct MonthApp {
    year: i32,
    month: u32,
    view: CalendarViewMode,
    selected: NaiveDate,
}

#[derive(Debug)]
pub enum MonthMsg {
    Previous,
    Next,
    CycleView,
    SelectMonth(u32),
    SelectYear(i32),
    Today,
    Select(NaiveDate),
    Close,
}

pub struct MonthWidgets {
    previous: gtk::Button,
    next: gtk::Button,
    grid: gtk::Grid,
    title_label: gtk::Label,
    title_icon: gtk::Image,
    selected_label: gtk::Label,
}

impl Component for MonthApp {
    type Init = ();
    type Input = MonthMsg;
    type Output = ();
    type CommandOutput = ();
    type Root = adw::ApplicationWindow;
    type Widgets = MonthWidgets;

    fn init_root() -> Self::Root {
        let root = adw::ApplicationWindow::builder()
            .title("Calendar")
            .default_width(400)
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
            view: CalendarViewMode::Days,
            selected: today,
        };

        let root_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root_box.add_css_class("panel");

        let topbar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        topbar.add_css_class("topbar");

        let previous = icon_button(
            "go-previous-symbolic",
            &["nav-button", "icon-button"],
            "Previous month",
        );
        let next = icon_button(
            "go-next-symbolic",
            &["nav-button", "icon-button"],
            "Next month",
        );
        let close = icon_button(
            "window-close-symbolic",
            &["close-button", "icon-button"],
            "Close",
        );
        let (title_button, title_label, title_icon) = calendar_title_button("");
        title_button.set_hexpand(true);

        topbar.append(&previous);
        topbar.append(&title_button);
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
            let sender = sender.clone();
            close.connect_clicked(move |_| sender.input(MonthMsg::Close));
        }
        {
            let sender = sender.clone();
            previous.connect_clicked(move |_| sender.input(MonthMsg::Previous));
        }
        {
            let sender = sender.clone();
            title_button.connect_clicked(move |_| sender.input(MonthMsg::CycleView));
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
        {
            let sender = sender.clone();
            add_escape_action(&root, move || sender.input(MonthMsg::Close));
        }

        let mut widgets = MonthWidgets {
            previous,
            next,
            grid,
            title_label,
            title_icon,
            selected_label,
        };
        render_month(&model, &mut widgets, sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            MonthMsg::Previous => {
                self.move_month(-1);
            }
            MonthMsg::Next => {
                self.move_month(1);
            }
            MonthMsg::CycleView => {
                self.view = self.view.next_level();
            }
            MonthMsg::SelectMonth(month) => {
                self.month = month;
                self.view = CalendarViewMode::Days;
            }
            MonthMsg::SelectYear(year) => {
                self.year = year;
                self.view = CalendarViewMode::Months;
            }
            MonthMsg::Today => {
                let today = Local::now().date_naive();
                self.year = today.year();
                self.month = today.month();
                self.view = CalendarViewMode::Days;
                self.selected = today;
            }
            MonthMsg::Select(day) => {
                self.selected = day;
                self.year = day.year();
                self.month = day.month();
                self.view = CalendarViewMode::Days;
            }
            MonthMsg::Close => {
                root.close();
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        render_month(self, widgets, sender);
    }
}

impl MonthApp {
    fn move_month(&mut self, delta: i32) {
        let month_delta = match self.view {
            CalendarViewMode::Days => delta,
            CalendarViewMode::Months => delta * 12,
            CalendarViewMode::Years => delta * YEAR_PAGE_STEP * 12,
        };
        let (year, month) = shift_month(self.year, self.month, month_delta);
        self.year = year;
        self.month = month;
    }
}

fn render_month(model: &MonthApp, widgets: &mut MonthWidgets, sender: ComponentSender<MonthApp>) {
    clear_grid(&widgets.grid);

    widgets.title_label.set_text(&title_text(model));
    widgets
        .title_icon
        .set_icon_name(Some(title_icon_name(model.view)));
    widgets
        .previous
        .set_tooltip_text(Some(previous_tooltip(model.view)));
    widgets
        .next
        .set_tooltip_text(Some(next_tooltip(model.view)));
    widgets
        .selected_label
        .set_text(&model.selected.format("%A, %B %-d, %Y").to_string());

    match model.view {
        CalendarViewMode::Days => render_day_grid(model, widgets, sender),
        CalendarViewMode::Months => render_month_grid(model, widgets, sender),
        CalendarViewMode::Years => render_year_grid(model, widgets, sender),
    }
}

fn render_day_grid(
    model: &MonthApp,
    widgets: &mut MonthWidgets,
    sender: ComponentSender<MonthApp>,
) {
    configure_day_grid(&widgets.grid);

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

fn render_month_grid(
    model: &MonthApp,
    widgets: &mut MonthWidgets,
    sender: ComponentSender<MonthApp>,
) {
    configure_picker_grid(&widgets.grid);
    let today = Local::now().date_naive();
    for month in 1..=12 {
        let index = month - 1;
        let item = classed_button(month_name(month), &["calendar-picker-cell"]);
        item.set_size_request(108, 44);
        if month == model.month {
            item.add_css_class("selected");
        }
        if today.year() == model.year && today.month() == month {
            item.add_css_class("today");
        }

        let sender = sender.clone();
        item.connect_clicked(move |_| sender.input(MonthMsg::SelectMonth(month)));

        widgets
            .grid
            .attach(&item, (index % 3) as i32, (index / 3) as i32, 1, 1);
    }
}

fn render_year_grid(
    model: &MonthApp,
    widgets: &mut MonthWidgets,
    sender: ComponentSender<MonthApp>,
) {
    configure_picker_grid(&widgets.grid);
    let today = Local::now().date_naive();
    let start = year_page_start(model.year);
    for offset in 0..YEAR_GRID_COUNT {
        let year = start + offset;
        let item = classed_button(&year.to_string(), &["calendar-picker-cell"]);
        item.set_size_request(108, 44);
        if year == model.year {
            item.add_css_class("selected");
        }
        if year == today.year() {
            item.add_css_class("today");
        }

        let sender = sender.clone();
        item.connect_clicked(move |_| sender.input(MonthMsg::SelectYear(year)));

        widgets.grid.attach(&item, offset % 3, offset / 3, 1, 1);
    }
}

fn configure_day_grid(grid: &gtk::Grid) {
    grid.remove_css_class("calendar-picker-grid");
    grid.set_column_spacing(6);
    grid.set_row_spacing(7);
    grid.set_halign(gtk::Align::Center);
}

fn configure_picker_grid(grid: &gtk::Grid) {
    grid.add_css_class("calendar-picker-grid");
    grid.set_column_spacing(8);
    grid.set_row_spacing(8);
    grid.set_halign(gtk::Align::Center);
}

fn calendar_title_button(text: &str) -> (gtk::Button, gtk::Label, gtk::Image) {
    let button = gtk::Button::new();
    button.add_css_class("calendar-title-button");
    button.set_tooltip_text(Some("Change calendar view"));

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    content.add_css_class("calendar-title-content");
    content.set_halign(gtk::Align::Center);
    content.set_valign(gtk::Align::Center);

    let text_label = label(text, &["title"], 0.5, false);
    content.append(&text_label);

    let icon = gtk::Image::from_icon_name("go-down-symbolic");
    icon.add_css_class("calendar-title-icon");
    content.append(&icon);

    button.set_child(Some(&content));
    (button, text_label, icon)
}

fn title_text(model: &MonthApp) -> String {
    match model.view {
        CalendarViewMode::Days => format!("{} {}", month_name(model.month), model.year),
        CalendarViewMode::Months => model.year.to_string(),
        CalendarViewMode::Years => {
            let start = year_page_start(model.year);
            format!("{}-{}", start, start + YEAR_GRID_COUNT - 1)
        }
    }
}

fn title_icon_name(view: CalendarViewMode) -> &'static str {
    if view == CalendarViewMode::Years {
        "go-up-symbolic"
    } else {
        "go-down-symbolic"
    }
}

fn previous_tooltip(view: CalendarViewMode) -> &'static str {
    match view {
        CalendarViewMode::Days => "Previous month",
        CalendarViewMode::Months => "Previous year",
        CalendarViewMode::Years => "Previous years",
    }
}

fn next_tooltip(view: CalendarViewMode) -> &'static str {
    match view {
        CalendarViewMode::Days => "Next month",
        CalendarViewMode::Months => "Next year",
        CalendarViewMode::Years => "Next years",
    }
}
