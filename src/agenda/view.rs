mod calendar;
mod cards;
mod list;
mod status;

use super::{AgendaApp, AgendaWidgets, auth_prompt};
use crate::calendar::date::event_days_for_timezone;
use crate::i18n::translate;
use crate::ui::clear_box;
use adw::prelude::*;
use relm4::ComponentSender;
use std::collections::BTreeSet;

pub(super) fn render(
    model: &AgendaApp,
    widgets: &mut AgendaWidgets,
    sender: ComponentSender<AgendaApp>,
) {
    clear_box(&widgets.content);
    if model.settings_open {
        widgets.content.append(&widgets.settings.panel);
        update_topbar(model, widgets);
        return;
    }

    let focus_auth_prompt = auth_prompt::should_focus(&model.state, model.authenticating);
    let calendar_event_days = if focus_auth_prompt {
        BTreeSet::new()
    } else {
        event_days_for_timezone(&model.state.events, model.query.timezone.as_deref())
    };

    widgets.content.append(&calendar::build(
        model,
        &calendar_event_days,
        sender.clone(),
    ));
    widgets.content.append(&list::build(model, sender));
    update_topbar(model, widgets);
}

fn update_topbar(model: &AgendaApp, widgets: &AgendaWidgets) {
    let lang = model.language();
    widgets
        .refresh
        .set_sensitive(!model.settings_open && !model.state.loading && !model.authenticating);
    widgets
        .refresh
        .set_tooltip_text(Some(if model.authenticating {
            translate(lang, "authenticating")
        } else if model.settings_open {
            translate(lang, "settings_open")
        } else if model.state.loading {
            translate(lang, "refreshing")
        } else {
            translate(lang, "refresh")
        }));
    let status = if model.settings_open {
        translate(lang, "settings").to_string()
    } else if model.authenticating {
        translate(lang, "authenticating").to_string()
    } else {
        status::agenda(&model.state, lang)
    };
    widgets.status_label.set_text(&status);
}
