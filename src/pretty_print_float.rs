use float_pretty_print::PrettyPrintFloat;

/// Used to prevent breaking rounding as explained in https://github.com/vi/float-pretty-print/issues/1
pub struct PrettyPrintFloatWithFallback(pub f64);

impl std::fmt::Display for PrettyPrintFloatWithFallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let w = f.width().unwrap_or(3);
        let p = f.precision().unwrap_or(12);
        let tmp = format!("{:w$.p$}", PrettyPrintFloat(self.0), w = w, p = p);
        let parse_back: Result<f64, _> = tmp.parse();
        match parse_back {
            Ok(x) if (x - self.0).abs() < f64::EPSILON => tmp.fmt(f),
            _ => self.0.fmt(f),
        }
    }
}
