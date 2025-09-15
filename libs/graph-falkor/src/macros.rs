#[macro_export]
macro_rules! stringy {
    ($input:expr) => {
        {
            format!("\"{}\"", $input)
        }
    };
}
