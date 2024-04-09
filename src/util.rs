pub mod color {
    use std::fmt::Write as _;
    use color_eyre::owo_colors::OwoColorize;

    pub fn diff(source: String) -> String {
        let mut out = String::new();

        for line in source.lines() {
            if line.starts_with("+") {
                write!(&mut out, "{}\n", line.green()).unwrap();
            } else if line.starts_with("-") {
                write!(&mut out, "{}\n", line.red()).unwrap();
            } else {
                write!(&mut out, "{line}\n").unwrap();
            }
        }

        out
    }
}

pub mod fmt {
    pub fn indent(source: String, n: usize) -> String {
        source
            .lines()
            .map(|line| format!("{}{}\n", " ".repeat(n), line))
            .collect()
    }
}
