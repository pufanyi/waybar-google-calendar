use crate::calendar::date::{month_dates, shift_month};
use crate::calendar::view::{
    CalendarViewMode, YEAR_GRID_COUNT, YEAR_PAGE_STEP, calendar_title_icon_name,
    calendar_title_text, next_calendar_tooltip, previous_calendar_tooltip, year_page_start,
};
use crate::i18n::{month_name, translate, weekday_short};
use crate::storage::settings::{Language, WeekStart, read_settings};
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
    language: Language,
    week_start: WeekStart,
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
        let settings = read_settings().unwrap_or_default();
        let language = settings.language.unwrap_or_default();
        let week_start = settings.week_start.unwrap_or_default();
        let model = MonthApp {
            year: today.year(),
            month: today.month(),
            view: CalendarViewMode::Days,
            selected: today,
            language,
            week_start,
        };

        let root_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root_box.add_css_class("panel");

        let topbar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        topbar.add_css_class("topbar");

        let previous = icon_button(
            "go-previous-symbolic",
            &["nav-button", "icon-button"],
            previous_calendar_tooltip(CalendarViewMode::Days, language),
        );
        let next = icon_button(
            "go-next-symbolic",
            &["nav-button", "icon-button"],
            next_calendar_tooltip(CalendarViewMode::Days, language),
        );
        let close = icon_button(
            "window-close-symbolic",
            &["close-button", "icon-button"],
            translate(language, "close"),
        );
        let (title_button, title_label, title_icon) = calendar_title_button("", language);
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
        let today_button = classed_button(translate(language, "today"), &["action-button"]);
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

    widgets.title_label.set_text(&calendar_title_text(
        model.view,
        model.year,
        model.month,
        model.language,
    ));
    widgets
        .title_icon
        .set_icon_name(Some(calendar_title_icon_name(model.view)));
    widgets
        .previous
        .set_tooltip_text(Some(previous_calendar_tooltip(model.view, model.language)));
    widgets
        .next
        .set_tooltip_text(Some(next_calendar_tooltip(model.view, model.language)));
    let selected_text = if model.language == Language::Chinese {
        model.selected.format("%Y-%m-%d").to_string()
    } else {
        model.selected.format("%A, %B %-d, %Y").to_string()
    };
    widgets.selected_label.set_text(&selected_text);

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

    for (col, weekday) in weekday_short(model.language, model.week_start)
        .iter()
        .enumerate()
    {
        let item = label(weekday, &["weekday"], 0.5, false);
        item.set_size_request(40, 22);
        widgets.grid.attach(&item, col as i32, 0, 1, 1);
    }

    let today = Local::now().date_naive();
    for (index, day) in month_dates(model.year, model.month, model.week_start)
        .iter()
        .enumerate()
    {
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
        let item = classed_button(month_name(model.language, month), &["calendar-picker-cell"]);
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

fn calendar_title_button(text: &str, language: Language) -> (gtk::Button, gtk::Label, gtk::Image) {
    let button = gtk::Button::new();
    button.set_cursor_from_name(Some("pointer"));
    button.add_css_class("calendar-title-button");
    button.set_tooltip_text(Some(translate(language, "change_calendar_view")));

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
