mod calendar;
mod cards;
mod list;
mod status;

use super::{AgendaApp, AgendaWidgets, auth_prompt};
use crate::calendar::date::event_days;
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
    let focus_auth_prompt = auth_prompt::should_focus(&model.state, model.authenticating);
    let calendar_event_days = if focus_auth_prompt {
        BTreeSet::new()
    } else {
        event_days(&model.state.events)
    };

    widgets.content.append(&calendar::build(
        model,
        &calendar_event_days,
        sender.clone(),
    ));
    widgets.content.append(&list::build(
        &model.query,
        &model.state,
        model.selected_day,
        model.authenticating,
        sender,
    ));
    update_topbar(model, widgets);
}

fn update_topbar(model: &AgendaApp, widgets: &AgendaWidgets) {
    widgets
        .refresh
        .set_sensitive(!model.state.loading && !model.authenticating);
    widgets.refresh.set_label(if model.authenticating {
        "Authenticating"
    } else if model.state.loading {
        "Refreshing"
    } else {
        "Refresh"
    });
    let status = if model.authenticating {
        "Authenticating".to_string()
    } else {
        status::agenda(&model.state)
    };
    widgets.status_label.set_text(&status);
}
