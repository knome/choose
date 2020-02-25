use crate::io::{BufWriter, Write};
use std::convert::TryInto;

use crate::config::Config;

pub type Range = (Option<u32>, Option<u32>);

#[derive(Debug)]
pub enum Choice {
    Field(u32),
    FieldRange(Range),
}

impl Choice {
    #[cfg_attr(feature = "flame_it", flame)]
    pub fn print_choice(
        &self,
        line: &String,
        config: &Config,
        handle: &mut BufWriter<std::io::StdoutLock>,
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
    fn get_choice_slice<'a>(
        &self,
        line: &'a String,
        config: &Config,
        handle: &mut BufWriter<std::io::StdoutLock>,
    ) {
        let words = config
            .separator
            .split(line)
            .into_iter()
            .filter(|s| !s.is_empty())
            .enumerate();

        match self {
            Choice::Field(i) => words
                .filter(|x| x.0 == *i as usize)
                .map(|x| x.1)
                .for_each(|x| write!(handle, "{} ", x).unwrap()),
            Choice::FieldRange(r) => match r {
                (None, None) => words
                    .map(|x| x.1)
                    .for_each(|x| write!(handle, "{} ", x).unwrap()),
                (Some(start), None) => words
                    .filter(|x| x.0 >= (*start).try_into().unwrap())
                    .map(|x| x.1)
                    .for_each(|x| write!(handle, "{} ", x).unwrap()),
                (None, Some(end)) => {
                    let e: usize = if config.opt.exclusive {
                        (end - 1).try_into().unwrap()
                    } else {
                        (*end).try_into().unwrap()
                    };
                    words
                        .filter(|x| x.0 <= e)
                        .map(|x| x.1)
                        .for_each(|x| write!(handle, "{} ", x).unwrap())
                }
                (Some(start), Some(end)) => {
                    let e: usize = if config.opt.exclusive {
                        (end - 1).try_into().unwrap()
                    } else {
                        (*end).try_into().unwrap()
                    };
                    words
                        .filter(|x| {
                            (x.0 <= e && x.0 >= (*start).try_into().unwrap())
                                || self.is_reverse_range()
                                    && (x.0 >= e && x.0 <= (*start).try_into().unwrap())
                        })
                        .map(|x| x.1)
                        .for_each(|x| write!(handle, "{} ", x).unwrap())
                }
            },
        };

        //if self.is_reverse_range() {
        //slices.reverse();
        //}

        //return slices;
    }
}

#[cfg(test)]
mod tests {

    use crate::config::{Config, Opt};
    use std::ffi::OsString;
    use structopt::StructOpt;

use std::io;

    impl Config {
        pub fn from_iter<I>(iter: I) -> Self
        where
            I: IntoIterator,
            I::Item: Into<OsString> + Clone,
        {
            return Config::new(Opt::from_iter(iter));
        }
    }

    mod get_choice_slice_tests {
        use super::*;

        #[test]
        fn print_0() {
let stdout = io::stdout();
let lock = stdout.lock();
let mut handle = io::BufWriter::new(lock);

            let config = Config::from_iter(vec!["choose", "0"]);
            assert_eq!(
                vec!["rust"],
                config.opt.choice[0]
                    .get_choice_slice(&String::from("rust is pretty cool"), &config, &mut handle)
            );
        }

        #[test]
        fn print_after_end() {
            let config = Config::from_iter(vec!["choose", "10"]);
            assert_eq!(
                Vec::<&str>::new(),
                config.opt.choice[0]
                    .get_choice_slice(&String::from("rust is pretty cool"), &config)
            );
        }

        #[test]
        fn print_out_of_order() {
            let config = Config::from_iter(vec!["choose", "3", "1"]);
            assert_eq!(
                vec!["cool"],
                config.opt.choice[0]
                    .get_choice_slice(&String::from("rust is pretty cool"), &config)
            );
            assert_eq!(
                vec!["is"],
                config.opt.choice[1]
                    .get_choice_slice(&String::from("rust is pretty cool"), &config)
            );
        }

        #[test]
        fn print_1_to_3_exclusive() {
            let config = Config::from_iter(vec!["choose", "1:3", "-x"]);
            assert_eq!(
                vec!["is", "pretty"],
                config.opt.choice[0]
                    .get_choice_slice(&String::from("rust is pretty cool"), &config)
            );
        }

        #[test]
        fn print_1_to_3() {
            let config = Config::from_iter(vec!["choose", "1:3"]);
            assert_eq!(
                vec!["is", "pretty", "cool"],
                config.opt.choice[0]
                    .get_choice_slice(&String::from("rust is pretty cool"), &config)
            );
        }

        #[test]
        fn print_1_to_3_separated_by_hashtag() {
            let config = Config::from_iter(vec!["choose", "1:3", "-f", "#"]);
            assert_eq!(
                vec!["is", "pretty", "cool"],
                config.opt.choice[0]
                    .get_choice_slice(&String::from("rust#is#pretty#cool"), &config)
            );
        }

        #[test]
        fn print_1_to_3_separated_by_varying_multiple_hashtag_exclusive() {
            let config = Config::from_iter(vec!["choose", "1:3", "-f", "#", "-x"]);
            assert_eq!(
                vec!["is", "pretty"],
                config.opt.choice[0]
                    .get_choice_slice(&String::from("rust##is###pretty####cool"), &config)
            );
        }

        #[test]
        fn print_1_to_3_separated_by_varying_multiple_hashtag() {
            let config = Config::from_iter(vec!["choose", "1:3", "-f", "#"]);
            assert_eq!(
                vec!["is", "pretty", "cool"],
                config.opt.choice[0]
                    .get_choice_slice(&String::from("rust##is###pretty####cool"), &config)
            );
        }

        #[test]
        fn print_1_to_3_separated_by_regex_group_vowels_exclusive() {
            let config = Config::from_iter(vec!["choose", "1:3", "-f", "[aeiou]", "-x"]);
            assert_eq!(
                vec![" q", "ck br"],
                config.opt.choice[0].get_choice_slice(
                    &String::from("the quick brown fox jumped over the lazy dog"),
                    &config
                )
            );
        }

        #[test]
        fn print_1_to_3_separated_by_regex_group_vowels() {
            let config = Config::from_iter(vec!["choose", "1:3", "-f", "[aeiou]"]);
            assert_eq!(
                vec![" q", "ck br", "wn f"],
                config.opt.choice[0].get_choice_slice(
                    &String::from("the quick brown fox jumped over the lazy dog"),
                    &config
                )
            );
        }

        #[test]
        fn print_3_to_1() {
            let config = Config::from_iter(vec!["choose", "3:1"]);
            assert_eq!(
                vec!["pretty", "is", "lang"],
                config.opt.choice[0]
                    .get_choice_slice(&String::from("rust lang is pretty darn cool"), &config)
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
