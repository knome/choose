use crate::config::Config;
use crate::io::{BufWriter, Write};

pub type Range = (Option<u32>, Option<u32>);

#[derive(Debug)]
pub enum Choice {
    Field(u32),
    FieldRange(Range),
}

impl Choice {
    #[cfg_attr(feature = "flame_it", flame)]
    pub fn print_choice<WriterType: Write>(
        &self,
        line: &String,
        config: &Config,
        handle: &mut BufWriter<WriterType>,
    ) {
        self.get_choice_slice(line, config, handle);
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn is_reverse_range(&self) -> bool {
        match self {
            Choice::Field(_) => false,
            Choice::FieldRange(r) => match r {
                (Some(start), Some(end)) => end < start,
                _ => false,
            },
        }
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn get_choice_slice<'a, WriterType: Write>(
        &self,
        line: &'a String,
        config: &Config,
        handle: &mut BufWriter<WriterType>,
    ) {
        let bounds = match self {
            Choice::Field(x) => (*x, *x),
            Choice::FieldRange(x) => match x {
                (None, None) => (0, u32::max_value()),
                (Some(b), None) => (*b, u32::max_value()),
                (None, Some(e)) => (0, *e),
                (Some(b), Some(e)) => (*b, *e),
            },
        };

        config
            .separator
            .split(line)
            .filter(|s| !s.is_empty())
            .enumerate()
            .filter(|(i, _)| {
                *i as u32 >= bounds.0
                    && (((*i as u32) <= bounds.1 && !config.opt.exclusive)
                        || ((*i as u32) < bounds.1 && config.opt.exclusive))
            })
            .for_each(|(_, word)| write!(handle, "{} ", word).unwrap());
    }
}

#[cfg(test)]
mod tests {

    use crate::config::{Config, Opt};
    use std::ffi::OsString;
    use std::io::{self, BufWriter, Write};
    use structopt::StructOpt;

    impl Config {
        pub fn from_iter<I>(iter: I) -> Self
        where
            I: IntoIterator,
            I::Item: Into<OsString> + Clone,
        {
            return Config::new(Opt::from_iter(iter));
        }
    }

    struct MockStdout {
        pub buffer: String,
    }

    impl MockStdout {
        fn new() -> Self {
            MockStdout {
                buffer: String::new(),
            }
        }

        fn str_from_buf_writer(b: BufWriter<MockStdout>) -> String {
            match b.into_inner() {
                Ok(b) => b.buffer,
                Err(_) => panic!("Failed to access BufWriter inner writer"),
            }
            .trim_end()
            .to_string()
        }
    }

    impl Write for MockStdout {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let mut bytes_written = 0;
            for i in buf {
                self.buffer.push(*i as char);
                bytes_written += 1;
            }
            Ok(bytes_written)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    mod get_choice_slice_tests {
        use super::*;

        #[test]
        fn print_0() {
            let config = Config::from_iter(vec!["choose", "0"]);
            let mut handle = BufWriter::new(MockStdout::new());

            config.opt.choice[0].get_choice_slice(
                &String::from("rust is pretty cool"),
                &config,
                &mut handle,
            );

            assert_eq!(
                String::from("rust"),
                MockStdout::str_from_buf_writer(handle)
            );
        }

        #[test]
        fn print_after_end() {
            let config = Config::from_iter(vec!["choose", "10"]);
            let mut handle = BufWriter::new(MockStdout::new());

            config.opt.choice[0].get_choice_slice(
                &String::from("rust is pretty cool"),
                &config,
                &mut handle,
            );

            assert_eq!(String::new(), MockStdout::str_from_buf_writer(handle));
        }

        #[test]
        fn print_out_of_order() {
            let config = Config::from_iter(vec!["choose", "3", "1"]);
            let mut handle = BufWriter::new(MockStdout::new());
            let mut handle1 = BufWriter::new(MockStdout::new());

            config.opt.choice[0].get_choice_slice(
                &String::from("rust is pretty cool"),
                &config,
                &mut handle,
            );

            assert_eq!(
                String::from("cool"),
                MockStdout::str_from_buf_writer(handle)
            );

            config.opt.choice[1].get_choice_slice(
                &String::from("rust is pretty cool"),
                &config,
                &mut handle1,
            );

            assert_eq!(String::from("is"), MockStdout::str_from_buf_writer(handle1));
        }

        #[test]
        fn print_1_to_3_exclusive() {
            let config = Config::from_iter(vec!["choose", "1:3", "-x"]);
            let mut handle = BufWriter::new(MockStdout::new());
            config.opt.choice[0].get_choice_slice(
                &String::from("rust is pretty cool"),
                &config,
                &mut handle,
            );
            assert_eq!(
                String::from("is pretty"),
                MockStdout::str_from_buf_writer(handle)
            );
        }

        #[test]
        fn print_1_to_3() {
            let config = Config::from_iter(vec!["choose", "1:3"]);
            let mut handle = BufWriter::new(MockStdout::new());
            config.opt.choice[0].get_choice_slice(
                &String::from("rust is pretty cool"),
                &config,
                &mut handle,
            );
            assert_eq!(
                String::from("is pretty cool"),
                MockStdout::str_from_buf_writer(handle)
            );
        }

        #[test]
        fn print_1_to_3_separated_by_hashtag() {
            let config = Config::from_iter(vec!["choose", "1:3", "-f", "#"]);
            let mut handle = BufWriter::new(MockStdout::new());
            config.opt.choice[0].get_choice_slice(
                &String::from("rust#is#pretty#cool"),
                &config,
                &mut handle,
            );
            assert_eq!(
                String::from("is pretty cool"),
                MockStdout::str_from_buf_writer(handle)
            );
        }

        #[test]
        fn print_1_to_3_separated_by_varying_multiple_hashtag_exclusive() {
            let config = Config::from_iter(vec!["choose", "1:3", "-f", "#", "-x"]);
            let mut handle = BufWriter::new(MockStdout::new());
            config.opt.choice[0].get_choice_slice(
                &String::from("rust##is###pretty####cool"),
                &config,
                &mut handle,
            );
            assert_eq!(
                String::from("is pretty"),
                MockStdout::str_from_buf_writer(handle)
            );
        }

        #[test]
        fn print_1_to_3_separated_by_varying_multiple_hashtag() {
            let config = Config::from_iter(vec!["choose", "1:3", "-f", "#"]);
            let mut handle = BufWriter::new(MockStdout::new());
            config.opt.choice[0].get_choice_slice(
                &String::from("rust##is###pretty####cool"),
                &config,
                &mut handle,
            );
            assert_eq!(
                String::from("is pretty cool"),
                MockStdout::str_from_buf_writer(handle)
            );
        }

        #[test]
        fn print_1_to_3_separated_by_regex_group_vowels_exclusive() {
            let config = Config::from_iter(vec!["choose", "1:3", "-f", "[aeiou]", "-x"]);
            let mut handle = BufWriter::new(MockStdout::new());
            config.opt.choice[0].get_choice_slice(
                &String::from("the quick brown fox jumped over the lazy dog"),
                &config,
                &mut handle,
            );
            assert_eq!(
                String::from(" q ck br"),
                MockStdout::str_from_buf_writer(handle)
            );
        }

        #[test]
        fn print_1_to_3_separated_by_regex_group_vowels() {
            let config = Config::from_iter(vec!["choose", "1:3", "-f", "[aeiou]"]);
            let mut handle = BufWriter::new(MockStdout::new());
            config.opt.choice[0].get_choice_slice(
                &String::from("the quick brown fox jumped over the lazy dog"),
                &config,
                &mut handle,
            );
            assert_eq!(
                String::from(" q ck br wn f"),
                MockStdout::str_from_buf_writer(handle)
            );
        }

        #[test]
        fn print_3_to_1() {
            let config = Config::from_iter(vec!["choose", "3:1"]);
            let mut handle = BufWriter::new(MockStdout::new());
            config.opt.choice[0].get_choice_slice(
                &String::from("rust lang is pretty darn cool"),
                &config,
                &mut handle,
            );
            assert_eq!(
                String::from("pretty is lang"),
                MockStdout::str_from_buf_writer(handle)
            );
        }

        #[test]
        fn print_3_to_beginning() {
            let config = Config::from_iter(vec!["choose", "3:"]);
            let mut handle = BufWriter::new(MockStdout::new());
            config.opt.choice[0].get_choice_slice(
                &String::from("rust lang is pretty darn cool"),
                &config,
                &mut handle,
            );
            assert_eq!(
                String::from("pretty is lang rust"),
                MockStdout::str_from_buf_writer(handle)
            );
        }

        #[test]
        fn print_end_to_1() {
            let config = Config::from_iter(vec!["choose", ":1"]);
            let mut handle = BufWriter::new(MockStdout::new());
            config.opt.choice[0].get_choice_slice(
                &String::from("rust lang is pretty darn cool"),
                &config,
                &mut handle,
            );
            assert_eq!(
                String::from("cool darn pretty is lang"),
                MockStdout::str_from_buf_writer(handle)
            );
        }
    }

    mod is_reverse_range_tests {
        use super::*;

        #[test]
        fn is_field_reversed() {
            let config = Config::from_iter(vec!["choose", "0"]);
            assert_eq!(false, config.opt.choice[0].is_reverse_range());
        }

        #[test]
        fn is_field_range_no_start_reversed() {
            let config = Config::from_iter(vec!["choose", ":2"]);
            assert_eq!(false, config.opt.choice[0].is_reverse_range());
        }

        #[test]
        fn is_field_range_no_end_reversed() {
            let config = Config::from_iter(vec!["choose", "2:"]);
            assert_eq!(false, config.opt.choice[0].is_reverse_range());
        }

        #[test]
        fn is_field_range_no_start_or_end_reversed() {
            let config = Config::from_iter(vec!["choose", ":"]);
            assert_eq!(false, config.opt.choice[0].is_reverse_range());
        }

        #[test]
        fn is_reversed_field_range_reversed() {
            let config = Config::from_iter(vec!["choose", "4:2"]);
            assert_eq!(true, config.opt.choice[0].is_reverse_range());
        }
    }
}
