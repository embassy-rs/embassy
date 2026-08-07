use alloc::format;
use alloc::vec::Vec;

use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Gauge, List, ListItem, Paragraph, RatatuiLogo, Sparkline, Wrap};
use ratatui::{Frame, symbols};
use tui_big_text::BigText;

use crate::{HEAP, SDRAM_SIZE};

pub struct Tui {
    count: u64,
}

impl Tui {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let root = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from(Vec::from([
                " ".into(),
                Span::styled("Mousefood", Style::new().light_cyan().bold()),
                "  ".into(),
                Span::styled("Portrait Demo ", Style::new().dark_gray()),
            ])));
        let inner = root.inner(area);
        frame.render_widget(root, area);

        let layout = Layout::vertical([
            Constraint::Length(3), // header
            Constraint::Length(6), // logo card
            Constraint::Length(8), // gauges
            Constraint::Min(10),   // body
            Constraint::Length(3), // footer
        ])
        .split(inner);

        self.draw_header(frame, layout[0]);
        self.draw_logo(frame, layout[1]);
        self.draw_gauges(frame, layout[2]);
        self.draw_body(frame, layout[3]);
        self.draw_footer(frame, layout[4]);

        self.count = self.count.wrapping_add(1);
    }

    pub fn draw_embassy(&self, frame: &mut Frame, area: Rect) {
        let bt = BigText::builder()
            .pixel_size(tui_big_text::PixelSize::Quadrant)
            .style(Style::new().light_cyan())
            .lines(Vec::from(["EMBASSY".into()]))
            .centered()
            .build();

        frame.render_widget(bt, area);
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let cols = Layout::horizontal([Constraint::Length(35), Constraint::Fill(1)]).split(area);

        let phase = match (self.count / 20) % 4 {
            0 => "CONNECT",
            1 => "LINK",
            2 => "DHCP",
            _ => "OFFLINE",
        };

        frame.render_widget(RatatuiLogo::small(), cols[0]);

        let text = Line::from(Vec::from([
            Span::styled("Status ", Style::new().gray()),
            Span::styled(phase, Style::new().green().bold()),
            Span::raw("   "),
            Span::styled("Frame ", Style::new().gray()),
            Span::styled(format!("{}", self.count), Style::new().yellow().bold()),
        ]));

        let p = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::bordered().border_type(BorderType::Rounded));
        frame.render_widget(p, cols[1]);
    }

    fn draw_logo(&self, frame: &mut Frame, area: Rect) {
        let cols = Layout::horizontal([Constraint::Length(35), Constraint::Fill(1)]).split(area);

        let logo_block = Block::bordered().title("Logo").border_type(BorderType::Rounded);
        let logo_inner = logo_block.inner(cols[0]);
        frame.render_widget(logo_block, cols[0]);

        self.draw_embassy(frame, logo_inner);

        let pulse = ["·", "••", "•••", "••••"][(self.count as usize / 8) % 4];

        let info = Paragraph::new(Vec::from([
            Line::from(Vec::from([
                Span::styled("Renderer", Style::new().light_cyan().bold()),
                Span::raw("  "),
                Span::styled("online", Style::new().green()),
            ])),
            Line::from(Vec::from([
                Span::styled("Display", Style::new().light_magenta().bold()),
                Span::raw("  "),
                Span::styled("480×800", Style::new().white()),
            ])),
            Line::from(Vec::from([
                Span::styled("Activity", Style::new().yellow().bold()),
                Span::raw("  "),
                Span::styled(pulse, Style::new().yellow()),
            ])),
        ]))
        .block(Block::bordered().title("Overview").border_type(BorderType::Rounded))
        .wrap(Wrap { trim: true });

        frame.render_widget(info, cols[1]);
    }

    fn draw_gauges(&self, frame: &mut Frame, area: Rect) {
        let rows = Layout::vertical([Constraint::Length(3), Constraint::Length(3), Constraint::Length(2)]).split(area);

        let progress_a = ((self.count * 3) % 100) as u16;
        let progress_b = ((self.count * 5 + 27) % 100) as u16;

        let g1 = Gauge::default()
            .block(Block::bordered().title("Progress").border_type(BorderType::Rounded))
            .gauge_style(Style::new().light_cyan().on_black().bold())
            .percent(progress_a);

        let g2 = Gauge::default()
            .block(Block::bordered().title("Load").border_type(BorderType::Rounded))
            .gauge_style(Style::new().light_green().on_black().bold())
            .percent(progress_b);

        let stats = Line::from(Vec::from([
            Span::styled("FPS ", Style::new().gray()),
            Span::styled(format!("{}", 48 + (self.count % 7)), Style::new().white().bold()),
            Span::raw("   "),
            Span::styled("TE ", Style::new().gray()),
            Span::styled("locked", Style::new().green().bold()),
            Span::raw("   "),
            Span::styled("Mode ", Style::new().gray()),
            Span::styled("running", Style::new().light_blue()),
        ]));

        frame.render_widget(g1, rows[0]);
        frame.render_widget(g2, rows[1]);
        frame.render_widget(Paragraph::new(stats).alignment(Alignment::Center), rows[2]);
    }

    fn draw_body(&self, frame: &mut Frame, area: Rect) {
        let cols = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).split(area);

        self.draw_activity(frame, cols[0]);
        self.draw_sidebar(frame, cols[1]);
    }

    fn draw_activity(&self, frame: &mut Frame, area: Rect) {
        let rows = Layout::vertical([Constraint::Min(8), Constraint::Length(7)]).split(area);

        let items = [
            "Ethernet Link UP",
            "Acquiring DHCP lease",
            "Connecting to HTTP",
            "Loading Configuration",
            "Sampling ADC sensors",
            "Sending sensor data",
            "Disconnecting",
            "Sleeping",
        ];

        let mut log: Vec<ListItem> = Vec::new();
        for i in 0..area.height as usize {
            let idx = ((self.count as usize) + i) % items.len();
            let prefix = "•";
            log.push(ListItem::new(Line::from(Vec::from([
                Span::styled(prefix, Style::new().light_green()),
                Span::raw(" "),
                Span::raw(items[idx]),
            ]))));
        }

        let list = List::new(log).block(Block::bordered().title("Activity").border_type(BorderType::Rounded));
        frame.render_widget(list, rows[0]);

        let data: [u64; 41] = core::array::from_fn(|i| {
            let t = self.count + i as u64;
            4 + ((t * 13 + (i as u64 * 7)) % 32)
        });

        let spark = Sparkline::default()
            .block(Block::bordered().title("Trend").border_type(BorderType::Rounded))
            .data(&data)
            .style(Style::new().light_yellow())
            .bar_set(symbols::bar::NINE_LEVELS);

        frame.render_widget(spark, rows[1]);
    }

    fn draw_sidebar(&self, frame: &mut Frame, area: Rect) {
        let rows = Layout::vertical([Constraint::Length(6), Constraint::Length(6), Constraint::Min(4)]).split(area);

        let memory = Paragraph::new(Vec::from([
            Line::from(Vec::from([
                Span::styled("SDRAM", Style::new().light_blue().bold()),
                Span::raw(format!(" {} MiB", SDRAM_SIZE / 1024 / 1024)),
            ])),
            Line::from(Vec::from([
                Span::styled("Heap", Style::new().light_cyan()),
                Span::raw(format!(" {} KiB used", HEAP.used() / 1024)),
            ])),
            Line::from(Vec::from([
                Span::styled("Heap", Style::new().light_cyan()),
                Span::raw(format!(" {} KiB free", HEAP.free() / 1024)),
            ])),
        ]))
        .block(Block::bordered().title("Memory").border_type(BorderType::Rounded));
        frame.render_widget(memory, rows[0]);

        let links = Paragraph::new(Vec::from([
            Line::from(Vec::from([
                Span::styled("Glass", Style::new().light_green().bold()),
                Span::raw(" 480x800 @ 24bpp"),
            ])),
            Line::from(Vec::from([
                Span::styled("DSI Link", Style::new().magenta().bold()),
                Span::raw(" 2 lanes @ 1 Gbps/lane"),
            ])),
            Line::from(Vec::from([
                Span::styled("LTDC", Style::new().yellow().bold()),
                Span::raw(" auto refresh"),
            ])),
            Line::from(Vec::from([
                Span::styled("Controller", Style::new().light_red().bold()),
                Span::raw(" NT35510"),
            ])),
        ]))
        .block(Block::bordered().title("Display").border_type(BorderType::Rounded));
        frame.render_widget(links, rows[1]);

        let tip = Paragraph::new("Ratatui demo running on an STM32H747I-DISCO using Embassy.")
            .block(Block::bordered().title("Note").border_type(BorderType::Rounded))
            .wrap(Wrap { trim: true });
        frame.render_widget(tip, rows[2]);
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = Paragraph::new(Line::from(Vec::from([
            Span::styled("▲/▼", Style::new().gray()),
            Span::raw(" navigate  "),
            Span::styled("Enter", Style::new().gray()),
            Span::raw(" select  "),
            Span::styled("q", Style::new().gray()),
            Span::raw(" quit"),
        ])))
        .alignment(Alignment::Center)
        .block(Block::bordered().border_type(BorderType::Rounded));

        frame.render_widget(footer, area);
    }
}
