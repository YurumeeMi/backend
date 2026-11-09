use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::{app::account::Account, ui::theme};

pub fn render(frame: &mut Frame, area: Rect, accounts: &[Account]) {
    let title = Row::new(vec![
        Cell::from("CODE"),
        Cell::from("ACCOUNT NAME"),
        Cell::from("TYPE"),
        Cell::from("OPENING BALANCE"),
        Cell::from("STATUS"),
    ])
    .style(
        Style::default()
            .fg(theme::TEXT_SECONDARY)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = accounts
        .iter()
        .map(|account| {
            let status = if account.is_active {
                Span::styled("Active", Style::default().fg(theme::SUCCESS))
            } else {
                Span::styled("Inactive", Style::default().fg(theme::DANGER))
            };

            Row::new(vec![
                Cell::from(account.code.clone()),
                Cell::from(account.name.clone()),
                Cell::from(account.account_type.clone()),
                Cell::from(account.opening_balance.clone()),
                Cell::from(status),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Percentage(30),
            Constraint::Length(15),
            Constraint::Length(20),
            Constraint::Length(12),
        ],
    )
    .header(title)
    .block(
        Block::default()
            .title(" CHART OF ACCOUNTS ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER)),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
}
