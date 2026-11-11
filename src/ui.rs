use std::io::{self, Write};

const GREEN: &str = "32";
const RED: &str = "31";

/// Console output for commands. Generic over the writer so tests can capture it.
pub struct Ui<W: Write> {
    out: W,
    color: bool,
}

impl<W: Write> Ui<W> {
    pub fn new(out: W, color: bool) -> Self {
        Self { out, color }
    }

    pub fn into_inner(self) -> W {
        self.out
    }

    pub fn start(&mut self, msg: &str) -> io::Result<()> {
        self.colored(GREEN, &format!("- {msg} -"))
    }

    pub fn note(&mut self, msg: &str) -> io::Result<()> {
        writeln!(self.out, "* {msg}")
    }

    pub fn warn(&mut self, msg: &str) -> io::Result<()> {
        self.colored(RED, msg)
    }

    pub fn say(&mut self, msg: &str) -> io::Result<()> {
        writeln!(self.out, "{msg}")
    }

    pub fn success(&mut self, msg: &str) -> io::Result<()> {
        self.colored(GREEN, &big_message(msg))
    }

    pub fn failure(&mut self, msg: &str) -> io::Result<()> {
        self.colored(RED, &big_message(msg))
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }

    fn colored(&mut self, code: &str, text: &str) -> io::Result<()> {
        if self.color {
            writeln!(self.out, "\x1b[{code}m{text}\x1b[0m")
        } else {
            writeln!(self.out, "{text}")
        }
    }
}

/// Wraps `msg` in a banner so it stands out from git's own output.
pub fn big_message(msg: &str) -> String {
    let spacer = format!(" {} ", "-".repeat(msg.chars().count() + 20));
    format!("{spacer}\n|          {msg}          |\n{spacer}")
}

/// "1 commit", "2 commits".
pub fn count(n: usize, noun: &str) -> String {
    let s = if n == 1 { "" } else { "s" };
    format!("{n} {noun}{s}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture(color: bool, f: impl FnOnce(&mut Ui<Vec<u8>>)) -> String {
        let mut ui = Ui::new(Vec::new(), color);
        f(&mut ui);
        String::from_utf8(ui.into_inner()).unwrap()
    }

    #[test]
    fn big_message_pads_to_message_width() {
        let lines: Vec<_> = big_message("hi").lines().map(str::to_owned).collect();
        assert_eq!(lines[0], format!(" {} ", "-".repeat(22)));
        assert_eq!(lines[1], "|          hi          |");
        assert_eq!(lines[0], lines[2]);
        assert_eq!(lines[0].len(), lines[1].len());
    }

    #[test]
    fn count_pluralises() {
        assert_eq!(count(0, "commit"), "0 commits");
        assert_eq!(count(1, "commit"), "1 commit");
        assert_eq!(count(2, "commit"), "2 commits");
    }

    #[test]
    fn plain_output_has_no_escapes() {
        let out = capture(false, |ui| {
            ui.start("go").unwrap();
            ui.note("hello").unwrap();
            ui.warn("careful").unwrap();
        });
        assert_eq!(out, "- go -\n* hello\ncareful\n");
    }

    #[test]
    fn colored_output_wraps_in_ansi() {
        let out = capture(true, |ui| ui.warn("bad").unwrap());
        assert_eq!(out, "\x1b[31mbad\x1b[0m\n");
    }
}
