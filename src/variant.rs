use std::collections::HashMap;
use std::convert::From;
use std::str::FromStr;
// use std::convert::From;

// use oxilangtag::{LanguageTag, LanguageTagParseError};
use once_cell::unsync::Lazy;
use regex::Regex;
use strum::{Display, EnumString, IntoStaticStr};

use crate::utils::get_with_fallback;

/// A Chinese variant parsed from a language tag
///
/// Currently supported variants are those listed in [Help:高级字词转换语法#组合转换标签](https://zh.wikipedia.org/wiki/Help:高级字词转换语法#组合转换标签).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
pub enum Variant {
    Zh,
    ZhHant,
    ZhHans,
    ZhTW,
    ZhHK,
    ZhMO,
    ZhMY,
    ZhSG,
    ZhCN,
    // Unknown(String)
}

/// Map variants to text, e.g. `zh-hans:计算机; zh-hant:電腦;`
#[derive(Debug, Clone)]
pub struct VariantMap(pub HashMap<Variant, String>);

impl VariantMap {
    /// Get the text for the target variant, if any
    #[inline(always)]
    pub fn get_text(&self, target: Variant) -> Option<&str> {
        self.0.get(&target).map(String::as_str)
    }

    /// Get the text for the target variant with automatic fallback
    ///
    /// It will panic if the inner map is empty itself
    pub fn get_text_with_fallback(&self, target: Variant) -> Option<&str> {
        // Ref: https://github.com/wikimedia/mediawiki/blob/6eda8891a0595e72e350998b6bada19d102a42d9/includes/language/converters/ZhConverter.php#L65
        use Variant::*;
        // self.0.

        match_fallback!(
            self.0,
            target,
            Zh -> [ZhHans, ZhHant, ZhCN, ZhTW, ZhHK, ZhSG, ZhMO, ZhMY],
            ZhHans -> [ ZhCN, ZhSG, ZhMY ],
            ZhHant -> [ ZhTW, ZhHK, ZhMO ],
            ZhCN -> [ ZhHans, ZhSG, ZhMY ],
            ZhSG -> [ ZhHans, ZhCN, ZhMY ],
            ZhMY -> [ ZhHans, ZhSG, ZhCN ],
            ZhTW -> [ ZhHant, ZhHK, ZhMO ],
            ZhHK -> [ ZhHant, ZhMO, ZhTW ],
            ZhMO -> [ ZhHant, ZhHK, ZhTW ],
        ) // FIX: TODO: falling back to zh finally?
          // match target {
          //     Zh => get_with_fallback!(self.0, Zh, ZhHans, ZhHant, ZhCN, ZhTW, ZhHK, ZhSG, ZhMO, ZhMY),
          //     ZhHant => get_with_fallback!(self.0, ZhHant, ZhTW, ZhHK, ZhMO),
          //     _ => None
          // }.map(String::as_ref)
          // define_fallback!(self,
          //     Variant::Zh => (Variant::Zh, )
          // )
          // unimplemented!();
    }

    /// Get the pairs of conversion for a target variant
    // TODO: better naming?
    pub fn get_convs_by_target(&self, target: Variant) -> Vec<(&str, &str)> {
        // MEDIAWIKI: unlike inline conversion rules, global conversion rule has no fallback
        if let Some(to) = self.0.get(&target) {
            let mut pairs = vec![];
            for (variant, from) in self.0.iter() {
                if *variant != target {
                    pairs.push((from.as_ref(), to.as_ref()));
                }
            }
            pairs
        } else {
            return vec![];
        }
    }
}

impl VariantMap {
    pub fn into_inner(self) -> HashMap<Variant, String> {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromStr for VariantMap {
    type Err = (); // TODO: better error propagation

    fn from_str(s: &str) -> Result<VariantMap, Self::Err> {
        let s = s.trim();
        let mut map = HashMap::new();
        // TODO: implement a clean iterator instead
        let mut parse_single = |s: &str| -> Result<(), Self::Err> {
            let (v, t) = s.split_at(s.find(':').ok_or(())?);
            map.insert(Variant::from_str(v).map_err(|_| ())?, t.to_owned());
            Ok(())
        };
        let mut i = 0;
        let mut ampersand = None;
        for (j, &c) in s.as_bytes().iter().enumerate() {
            match c {
                b'&' => {
                    ampersand = Some(j);
                    // if ampersand, the new & is the new start
                }
                b';' => {
                    if !(ampersand.is_some() && j - ampersand.unwrap() > 1) {
                        parse_single(&s[i..j])?;
                        i = j + 1;
                    }
                }
                _ => {
                    if ampersand.is_some() & !(b'#' == c || char::from(c).is_ascii_alphanumeric()) {
                        ampersand = None;
                    }
                }
            }
            // match &s[i]
        }
        if i != s.as_bytes().len() {
            parse_single(&s[i..])?;
        }

        // let p: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\w]+):()").unwrap());
        // TODO: more robust parser?
        // let hep: Lazy<Regex> = Lazy::new(|| Regex::new(r"(&[#a-zA-Z0-9]+);").unwrap());  // html entities
        // let mut es = hep.find_iter(s).map(|m| m.end() - 1).peekable(); // the semicolon indices of entites

        // let mut i = s.find(';')

        // for m in  {
        //     m.end() - 1;
        // }
        // s.split(";")
        Ok(VariantMap(map))
    }
}

impl From<HashMap<Variant, String>> for VariantMap {
    fn from(hm: HashMap<Variant, String>) -> Self {
        Self(hm)
    }
}

// Ref: https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=cdab97d0a7f71d9a13568c97ad3faf3a
macro_rules! match_fallback {
    ( $map:expr, $target:expr, $($t:tt)* ) => {
        match_fallback!(@build $map, $target, (), $($t)*)
        // match $target {
        //     match_fallback!(@build $map, $target, $($t)*)
        // }
    };
    (@build $map:expr, $target:expr, ($($arms:tt)*), $variant:ident -> [ $($fallbacks:tt)* ], $($others:tt)* ) => {
        // $variant => get_with_fallback!($map, $target, $($fallbacks)*).map(String::as_str()),
        match_fallback!(@build $map, $target, ($($arms)* $variant => get_with_fallback!($map, $variant, $($fallbacks)*),), $($others)*)
    };
    (@build $map:expr, $target:expr, ($($arms:tt)*) $(,)? ) => {
        match $target {
            $($arms)*
        }.map(String::as_str)
    };
}
use match_fallback;
