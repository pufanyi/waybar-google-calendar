use super::cards;
use crate::agenda::{AgendaApp, AgendaEditor, AgendaMsg};
use crate::calendar::date::{
    format_day_label_for_timezone, format_time_label_for_timezone, now_parts_for_timezone,
    parse_event_start_for_timezone,
};
use crate::calendar::model::{Event, EventKey, EventMutation};
use crate::i18n::translate;
use crate::storage::settings::Language;
use crate::ui::{DateTimePicker, classed_button, icon_button, label};
use adw::prelude::*;
use chrono::{Duration as ChronoDuration, NaiveDate, NaiveTime, Timelike};
use relm4::ComponentSender;

pub(super) fn build(model: &AgendaApp, sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let lang = model.language();
    let timezone = model.query.timezone.as_deref();

    match &model.event_editor {
        AgendaEditor::None => gtk::Box::new(gtk::Orientation::Vertical, 0),
        AgendaEditor::Add => event_form(
            translate(lang, "add_event"),
            FormSeed::new(model.selected_day, timezone),
            None,
            model.event_editor_msg.as_deref(),
            model.mutating_event,
            lang,
            sender,
        ),
        AgendaEditor::Detail(key) => model
            .event_by_key(key)
            .map(|event| {
                event_detail(
                    event,
                    timezone,
                    model.event_editor_msg.as_deref(),
                    model.mutating_event,
                    lang,
                    sender.clone(),
                )
            })
            .unwrap_or_else(|| missing_event(lang, sender)),
        AgendaEditor::Edit(key) => model
            .event_by_key(key)
            .map(|event| {
                event_form(
                    translate(lang, "edit_event"),
                    FormSeed::from_event(event, timezone),
                    event.key(),
                    model.event_editor_msg.as_deref(),
                    model.mutating_event,
                    lang,
                    sender.clone(),
                )
            })
            .unwrap_or_else(|| missing_event(lang, sender)),
        AgendaEditor::ConfirmDelete(key) => model
            .event_by_key(key)
            .map(|event| {
                confirm_delete(
                    event,
                    timezone,
                    model.event_editor_msg.as_deref(),
                    model.mutating_event,
                    lang,
                    sender.clone(),
                )
            })
            .unwrap_or_else(|| missing_event(lang, sender)),
    }
}

struct FormSeed {
    summary: String,
    location: String,
    description: String,
    start_date: NaiveDate,
    start_time: NaiveTime,
    end_date: NaiveDate,
    end_time: NaiveTime,
    all_day: bool,
}

impl FormSeed {
    fn new(selected_day: Option<NaiveDate>, timezone: Option<&str>) -> Self {
        let (start_date, start_time) = default_start(selected_day, timezone);
        let end = start_date.and_time(start_time) + ChronoDuration::hours(1);
        Self {
            summary: String::new(),
            location: String::new(),
            description: String::new(),
            start_date,
            start_time,
            end_date: end.date(),
            end_time: end.time(),
            all_day: false,
        }
    }

    fn from_event(event: &Event, timezone: Option<&str>) -> Self {
        let start = parse_event_start_for_timezone(&event.start, timezone);
        let end = parse_event_start_for_timezone(&event.end, timezone);
        let all_day = matches!(start, Some((_, None)));
        let start_date = start
            .map(|(date, _)| date)
            .unwrap_or_else(|| now_parts_for_timezone(timezone).0);
        let start_time = start
            .and_then(|(_, time)| time)
            .unwrap_or_else(|| NaiveTime::from_hms_opt(9, 0, 0).expect("valid fallback time"));
        let default_end = start_date.and_time(start_time) + ChronoDuration::hours(1);
        let end_date = end
            .map(|(date, time)| {
                if time.is_none() && date > start_date {
                    date - ChronoDuration::days(1)
                } else {
                    date
                }
            })
            .unwrap_or_else(|| default_end.date());
        let end_time = end
            .and_then(|(_, time)| time)
            .unwrap_or_else(|| default_end.time());

        Self {
            summary: event.summary.clone(),
            location: event.location.clone(),
            description: event.description.clone(),
            start_date,
            start_time,
            end_date,
            end_time,
            all_day,
        }
    }
}

fn default_start(
    selected_day: Option<NaiveDate>,
    timezone: Option<&str>,
) -> (NaiveDate, NaiveTime) {
    let (today, now) = now_parts_for_timezone(timezone);
    let date = selected_day.unwrap_or(today);
    if date != today {
        return (
            date,
            NaiveTime::from_hms_opt(9, 0, 0).expect("valid fallback time"),
        );
    }

    let now = now.with_second(0).unwrap_or(now);
    let minutes_to_add = 30 - (now.minute() % 30);
    let start = date.and_time(now) + ChronoDuration::minutes(minutes_to_add as i64);
    (start.date(), start.time())
}

fn event_detail(
    event: &Event,
    timezone: Option<&str>,
    message: Option<&str>,
    mutating: bool,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let panel = panel(translate(lang, "event_details"), lang, sender.clone());
    let content = gtk::Box::new(gtk::Orientation::Vertical, 10);
    content.add_css_class("event-detail-content");

    content.append(&label(
        event_summary(event, lang),
        &["event-detail-title"],
        0.0,
        true,
    ));
    content.append(&detail_row(
        translate(lang, "time"),
        &format_time_detail(event, timezone, lang),
        false,
    ));
    content.append(&detail_row(
        translate(lang, "calendar"),
        calendar_name(event, lang),
        false,
    ));
    if !event.location.trim().is_empty() {
        content.append(&detail_row(
            translate(lang, "location"),
            event.location.trim(),
            true,
        ));
    }
    if !event.description.trim().is_empty() {
        content.append(&detail_row(
            translate(lang, "description"),
            event.description.trim(),
            true,
        ));
    }
    if let Some(message) = message {
        content.append(&label(message, &["muted", "settings-message"], 0.0, true));
    }

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("event-editor-actions");
    let back = classed_button(translate(lang, "back"), &["action-button"]);
    {
        let sender = sender.clone();
        back.connect_clicked(move |_| sender.input(AgendaMsg::CloseEventEditor));
    }
    actions.append(&back);

    if !event.html_link.trim().is_empty() {
        let open = classed_button(translate(lang, "open_in_calendar"), &["action-button"]);
        let link = event.html_link.clone();
        let sender = sender.clone();
        open.connect_clicked(move |_| sender.input(AgendaMsg::OpenEventLink(link.clone())));
        actions.append(&open);
    }

    if let Some(key) = event.key() {
        let edit = classed_button(translate(lang, "edit"), &["action-button"]);
        {
            let sender = sender.clone();
            let key = key.clone();
            edit.connect_clicked(move |_| sender.input(AgendaMsg::EditEvent(key.clone())));
        }
        actions.append(&edit);

        let delete = classed_button(translate(lang, "delete_event"), &["action-button"]);
        delete.add_css_class("danger-action");
        delete.set_sensitive(!mutating);
        {
            let sender = sender.clone();
            delete.connect_clicked(move |_| sender.input(AgendaMsg::ConfirmDelete(key.clone())));
        }
        actions.append(&delete);
    }

    content.append(&actions);
    panel.append(&content);
    panel
}

fn confirm_delete(
    event: &Event,
    timezone: Option<&str>,
    message: Option<&str>,
    mutating: bool,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let panel = panel(
        translate(lang, "confirm_delete_event"),
        lang,
        sender.clone(),
    );
    let content = gtk::Box::new(gtk::Orientation::Vertical, 10);
    content.add_css_class("event-detail-content");
    content.append(&label(
        event_summary(event, lang),
        &["event-detail-title"],
        0.0,
        true,
    ));
    content.append(&detail_row(
        translate(lang, "time"),
        &format_time_detail(event, timezone, lang),
        false,
    ));
    content.append(&label(
        translate(lang, "delete_event_warning"),
        &["muted"],
        0.0,
        true,
    ));
    if let Some(message) = message {
        content.append(&label(message, &["muted", "settings-message"], 0.0, true));
    }

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("event-editor-actions");
    let cancel = classed_button(translate(lang, "cancel"), &["action-button"]);
    {
        let sender = sender.clone();
        let key = event.key();
        cancel.connect_clicked(move |_| {
            if let Some(key) = key.clone() {
                sender.input(AgendaMsg::ShowEventDetail(key));
            } else {
                sender.input(AgendaMsg::CloseEventEditor);
            }
        });
    }
    actions.append(&cancel);

    if let Some(key) = event.key() {
        let delete = classed_button(translate(lang, "delete_event"), &["action-button"]);
        delete.add_css_class("danger-action");
        delete.add_css_class("destructive-action");
        delete.set_sensitive(!mutating);
        delete.connect_clicked(move |_| sender.input(AgendaMsg::DeleteEvent(key.clone())));
        actions.append(&delete);
    }

    content.append(&actions);
    panel.append(&content);
    panel
}

fn event_form(
    title: &str,
    seed: FormSeed,
    key: Option<EventKey>,
    message: Option<&str>,
    mutating: bool,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let panel = panel(title, lang, sender.clone());
    let form = gtk::Box::new(gtk::Orientation::Vertical, 10);
    form.add_css_class("event-form");

    let summary = entry(&seed.summary, translate(lang, "event_title"));
    form.append(&form_row(translate(lang, "event_title"), &summary));

    let all_day = gtk::CheckButton::with_label(translate(lang, "all_day_event"));
    all_day.add_css_class("event-check");
    all_day.set_active(seed.all_day);
    form.append(&form_row(translate(lang, "all_day"), &all_day));

    let start = DateTimePicker::new(seed.start_date, seed.start_time);
    let end = DateTimePicker::new(seed.end_date, seed.end_time);
    start.set_time_sensitive(!seed.all_day);
    end.set_time_sensitive(!seed.all_day);

    {
        let start = start.clone();
        let end = end.clone();
        all_day.connect_toggled(move |button| {
            let timed = !button.is_active();
            start.set_time_sensitive(timed);
            end.set_time_sensitive(timed);
        });
    }

    form.append(&form_row(translate(lang, "start"), &start.container));
    form.append(&form_row(translate(lang, "end"), &end.container));

    let location = entry(&seed.location, translate(lang, "location"));
    form.append(&form_row(translate(lang, "location"), &location));

    let description = gtk::TextView::new();
    description.add_css_class("text-entry");
    description.add_css_class("event-description-entry");
    description.set_wrap_mode(gtk::WrapMode::WordChar);
    description.buffer().set_text(&seed.description);
    form.append(&form_row(translate(lang, "description"), &description));

    if let Some(message) = message {
        form.append(&label(message, &["muted", "settings-message"], 0.0, true));
    }

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("event-editor-actions");
    let cancel = classed_button(translate(lang, "cancel"), &["action-button"]);
    {
        let sender = sender.clone();
        cancel.connect_clicked(move |_| sender.input(AgendaMsg::CloseEventEditor));
    }
    actions.append(&cancel);

    let save = classed_button(
        if key.is_some() {
            translate(lang, "update_event")
        } else {
            translate(lang, "create_event")
        },
        &["action-button"],
    );
    save.add_css_class("primary-action");
    save.set_sensitive(!mutating);
    {
        let sender = sender.clone();
        save.connect_clicked(move |_| {
            let changes = EventMutation {
                summary: summary.text().to_string(),
                location: location.text().to_string(),
                description: text_view_content(&description),
                start_date: start.date_string(),
                start_time: start.time_string(),
                end_date: end.date_string(),
                end_time: end.time_string(),
                all_day: all_day.is_active(),
            };
            if let Some(key) = key.clone() {
                sender.input(AgendaMsg::UpdateEvent(key, changes));
            } else {
                sender.input(AgendaMsg::CreateEvent(changes));
            }
        });
    }
    actions.append(&save);
    form.append(&actions);

    panel.append(&form);
    panel
}

fn panel(title: &str, lang: Language, sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let panel = gtk::Box::new(gtk::Orientation::Vertical, 12);
    panel.add_css_class("event-editor-panel");

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.add_css_class("event-editor-header");
    header.append(&label(title, &["agenda-header"], 0.0, false));
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    header.append(&spacer);
    let close = icon_button(
        "go-previous-symbolic",
        &["action-button", "icon-button"],
        translate(lang, "back"),
    );
    close.connect_clicked(move |_| sender.input(AgendaMsg::CloseEventEditor));
    header.append(&close);
    panel.append(&header);

    panel
}

fn form_row(label_text: &str, input: &impl IsA<gtk::Widget>) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("event-form-row");
    let field_label = label(label_text, &["field-label"], 0.0, false);
    field_label.set_size_request(110, -1);
    row.append(&field_label);
    input.as_ref().set_hexpand(true);
    row.append(input);
    row
}

fn detail_row(title: &str, value: &str, wrap: bool) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("event-detail-row");
    let title = label(title, &["field-label"], 0.0, false);
    title.set_size_request(92, -1);
    row.append(&title);
    row.append(&label(value, &["muted"], 0.0, wrap));
    row
}

fn entry(text: &str, placeholder: &str) -> gtk::Entry {
    let entry = gtk::Entry::builder()
        .text(text)
        .placeholder_text(placeholder)
        .build();
    entry.add_css_class("text-entry");
    entry
}

fn text_view_content(view: &gtk::TextView) -> String {
    let buffer = view.buffer();
    buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), true)
        .to_string()
}

fn missing_event(lang: Language, sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let panel = panel(translate(lang, "event_details"), lang, sender);
    panel.append(&cards::message(
        translate(lang, "event_not_found"),
        Some(translate(lang, "event_not_found_detail")),
        false,
    ));
    panel
}

fn format_time_detail(event: &Event, timezone: Option<&str>, lang: Language) -> String {
    let date = parse_event_start_for_timezone(&event.start, timezone)
        .map(|(date, _)| format_day_label_for_timezone(date, timezone, lang))
        .unwrap_or_else(|| translate(lang, "time_unavailable").to_string());
    let time = format_time_label_for_timezone(&event.start, &event.end, timezone, lang);
    format!("{date} · {time}")
}

fn event_summary(event: &Event, lang: Language) -> &str {
    if event.summary.trim().is_empty() {
        translate(lang, "untitled_event")
    } else {
        event.summary.trim()
    }
}

fn calendar_name(event: &Event, lang: Language) -> &str {
    if event.calendar.trim().is_empty() {
        translate(lang, "calendar")
    } else {
        event.calendar.trim()
    }
}
