macro_rules! style {
    (RESET) => {
        "\x1b[0m"
    };
    (BOLD) => {
        "\x1b[1m"
    };
    (DIM) => {
        "\x1b[2m"
    };
    (ITALIC) => {
        "\x1b[3m"
    };
    (RED) => {
        "\x1b[31m"
    };
    (YELLOW) => {
        "\x1b[33m"
    };
    (CYAN) => {
        "\x1b[36m"
    };
    (MAGENTA) => {
        "\x1b[35m"
    };
    (WHITE) => {
        "\x1b[37m"
    };

    ($a:ident, $b:ident) => {
        concat!(style!($a), style!($b))
    };
    ($a:ident, $b:ident, $c:ident) => {
        concat!(style!($a), style!($b), style!($c))
    };
}

pub(super) const RESET: &str = style!(RESET);
pub(super) const BOLD: &str = style!(BOLD);
pub(super) const DIM: &str = style!(DIM);
pub(super) const ITALIC: &str = style!(ITALIC);
pub(super) const RED: &str = style!(RED);
pub(super) const YELLOW: &str = style!(YELLOW);
pub(super) const CYAN: &str = style!(CYAN);
pub(super) const MAGENTA: &str = style!(MAGENTA);
pub(super) const WHITE: &str = style!(WHITE);

pub(crate) fn restore_color(s: &str, base_color: &str) -> String {
    s.replace(style!(RESET), &format!("{}{}", style!(RESET), base_color))
}
