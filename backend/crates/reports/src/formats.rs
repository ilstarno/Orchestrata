use rust_xlsxwriter::{Format, FormatAlign};

pub(crate) const NAVY: u32 = 0x14253A;
pub(crate) const TEAL: u32 = 0x146D62;
pub(crate) const INK: u32 = 0x182D43;
const MUTED: u32 = 0x435972;
const WHITE: u32 = 0xFFFFFF;
const PALE: u32 = 0xF0F5FA;
const GREEN_PALE: u32 = 0xE6F4F1;
const AMBER_PALE: u32 = 0xFFF1DC;
const AMBER_INK: u32 = 0x795117;

const DATE_FORMAT: &str = "yyyy-mm-dd";
const INTEGER_FORMAT: &str = "#,##0";

pub(crate) struct Formats {
    pub masthead: Format,
    pub title: Format,
    pub subtitle: Format,
    pub metric_label: Format,
    pub metric_value: Format,
    pub header: Format,
    pub data: [Format; 2],
    pub date: [Format; 2],
    pub number: [Format; 2],
    pub status: [Format; 2],
}

impl Formats {
    pub fn new() -> Self {
        let title = Format::new()
            .set_background_color(NAVY)
            .set_font_color(WHITE)
            .set_bold()
            .set_font_size(22)
            .set_text_wrap()
            .set_align(FormatAlign::VerticalCenter)
            .set_indent(1);
        let masthead = title.clone().set_font_size(10).set_font_color(0xBDE7DE);
        let subtitle = Format::new()
            .set_font_color(MUTED)
            .set_text_wrap()
            .set_align(FormatAlign::VerticalCenter);
        let metric_label = row_format(PALE).set_bold().set_font_color(MUTED);
        let metric_value = metric_label
            .clone()
            .set_font_size(24)
            .set_font_color(NAVY)
            .set_num_format(INTEGER_FORMAT)
            .set_align(FormatAlign::Left);
        let header = row_format(TEAL).set_font_color(WHITE).set_bold();
        let data = [row_format(WHITE), row_format(PALE)];
        let date = data.clone().map(|format| {
            format
                .set_num_format(DATE_FORMAT)
                .set_align(FormatAlign::Center)
        });
        let number = data
            .clone()
            .map(|format| format.set_num_format(INTEGER_FORMAT));
        let status = [
            row_format(AMBER_PALE).set_font_color(AMBER_INK).set_bold(),
            row_format(GREEN_PALE).set_font_color(TEAL).set_bold(),
        ];
        Self {
            masthead,
            title,
            subtitle,
            metric_label,
            metric_value,
            header,
            data,
            date,
            number,
            status,
        }
    }
}

fn row_format(background: u32) -> Format {
    Format::new()
        .set_background_color(background)
        .set_font_color(INK)
        .set_text_wrap()
        .set_align(FormatAlign::VerticalCenter)
        .set_indent(1)
}
