//! An implementation of [UAX #14], also called the `Unicode Line Breaking
//! Algorithm`.
//!
//! [UAX #14]: https://www.unicode.org/reports/tr14/
use std::char;
use std::iter::Peekable;
use std::str::Chars;

mod generated;

pub use generated::convert_to_break_class;

/// A [Line Breaking Class].
///
/// Interacting directly with Line Breaking Classes is usually not neccessary
/// unless you wanted to implement something similar to [`BreakInfo`]. Missing
/// here are [SG] (invalid in any input), [SA] (treated as [CM] or [AL]
/// depending on its General Category), [CJ] (treated as [NS]), [XX], [SG] and
/// [AI] (all treated as [AL]). For more information see [LB1].
///
/// [SG]: https://www.unicode.org/reports/tr14/#SG
/// [SA]: https://www.unicode.org/reports/tr14/#SA
/// [CM]: https://www.unicode.org/reports/tr14/#CM
/// [AL]: https://www.unicode.org/reports/tr14/#AL
/// [CJ]: https://www.unicode.org/reports/tr14/#CJ
/// [NS]: https://www.unicode.org/reports/tr14/#NS
/// [XX]: https://www.unicode.org/reports/tr14/#XX
/// [SG]: https://www.unicode.org/reports/tr14/#SG
/// [AI]: https://www.unicode.org/reports/tr14/#AI
/// [AL]: https://www.unicode.org/reports/tr14/#AL
/// [Line Breaking Class]: https://www.unicode.org/reports/tr14/#Table1
/// [LB1]: https://www.unicode.org/reports/tr14/#LB1
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Class {
    // Non-tailorable Line Breaking Classes
    BK, // Mandatory Break
    CR, // Carriage Return
    LF, // Line Feed
    CM, // Combining Mark
    NL, // Next Line
    // SG,  // Surrogate - Not used
    WJ,  // Word Joiner
    ZW,  // Zero Width Space
    GL,  // Non-breaking ("Glue")
    SP,  // Space
    ZWJ, // Zero Width Joiner

    // Break Opportunities
    B2, // Break Opportunity Before and After
    BA, // Break After
    BB, // Break Before
    HY, // Hyphen
    CB, // Contingent Break After

    // Characters Prohibiting Certain Breaks
    CL, // Clone Punctuation
    CP, // Close Parenthesis
    EX, // Exclamation/Interrogation
    IN, // Inseparable
    NS, // Nonstarter
    OP, // Open Punctuation
    QU, // Quatation

    // Numeric Context
    IS, // Infix Numeric Separator
    NU, // Numeric
    PO, // Postfix Numeric
    PR, // Prefix Numeric
    SY, // Symbols Allowing Break After

    // Other Characters
    // AI, // Ambiguous (Alphabetic or Ideographic) - Not used
    AL, // Alphabetic
    // CJ, // Conditional Japanese Starter - Not used
    EB, // Emoji Base
    EM, // Emoji Modifier
    H2, // Hangul LV Syllable
    H3, // Hangul LVT Syllable
    HL, // Hebrew Letter
    ID, // Ideographic
    JL, // Hangul L Jamo
    JV, // Hangul V Jamo
    JT, // Hangul T Jamo
    RI, // Regional Indicator
    // SA, // Complex Context Dependent (South East Asian) - Not used
    XX, // Unknown
}

/// Used to specify whether a break is allowed or not.
///
/// `Mandatory` is where it is expected to be a line break, `Opportunity` is
/// where it is allowed to be a line break and `Prohibited` is where a line
/// break isn't allowed.
#[derive(Debug, PartialEq)]
pub enum Break {
    Mandatory,
    Opportunity,
    Prohibited,
}

/// An `Iterator` that provides information about possible line breaks in a
/// `str`.
///
/// As it checks the position after every `char`, it will not give
/// information about the position before the very first `char`. Luckily that
/// case is trivial as a line break is never allowed there.
///
/// # Examples
///
/// ```
/// use uax_14::{Break, BreakInfo};
///
/// let input = "Which is tree? 木禾夫🤔";
/// let mut split_input = String::new();
/// for (c, br) in BreakInfo::new(&input) {
///     split_input.push(c);
///     if br == Break::Mandatory || br == Break::Opportunity {
///         split_input.push('\n');
///     }
/// }
/// let lines = split_input.split('\n').collect::<Vec<&str>>();
/// assert_eq!(
///     lines,
///     ["Which ", "is ", "tree? ", "木", "禾", "夫", "🤔", ""]
/// );
/// ```
pub struct BreakInfo<'a> {
    iter: Peekable<Chars<'a>>,
    ri_count: usize,
    class_before_spaces: Option<Class>,
    next_is_prohibited: bool,
    treat_next_n1_as: Option<Class>,
}

/// Construct a `BreakInfo` from a `&str`.
impl<'a> BreakInfo<'a> {
    #[inline]
    pub fn new(input: &'a str) -> BreakInfo<'a> {
        BreakInfo {
            iter: input.chars().peekable(),
            ri_count: 0,
            class_before_spaces: None,
            next_is_prohibited: false,
            treat_next_n1_as: None,
        }
    }

    fn get_break(&mut self, mut n1: Class, n2: Class) -> Break {
        if self.next_is_prohibited {
            self.next_is_prohibited = false;
            return Break::Prohibited;
        }

        if let Some(c) = self.treat_next_n1_as {
            if c != Class::ZWJ {
                n1 = c;
            }
            self.treat_next_n1_as = None;
        }

        // LB30a uses the amount of RI in a row to determine if breaks are allowed

        // Special case when RI is the first character
        if self.ri_count == 0 && n1 == Class::RI {
            self.ri_count = 1;
        }
        if n2 == Class::RI {
            self.ri_count += 1;
        } else {
            self.ri_count = 0;
        }

        // LB8, LB14, LB15, LB16, LB17 all need to keep track of characters before
        // spaces.
        if n1 != Class::SP && n2 == Class::SP {
            self.class_before_spaces = Some(n1);
        }

        // LB10
        if n1 == Class::CM {
            n1 = Class::AL;
        }

        let b = match (n1, n2) {
            // LB4
            (Class::BK, _) => Break::Mandatory,

            // LB5
            (Class::CR, Class::LF) => Break::Prohibited,
            (Class::CR, _) | (Class::LF, _) | (Class::NL, _) => Break::Mandatory,

            // LB6
            (_, Class::BK) | (_, Class::CR) | (_, Class::LF) | (_, Class::NL) => Break::Prohibited,

            // LB7
            (_, Class::SP) | (_, Class::ZW) => Break::Prohibited,

            // LB8
            (Class::ZW, _) => Break::Opportunity,
            (Class::SP, _) if self.class_before_spaces == Some(Class::ZW) => Break::Opportunity,

            // LB8a
            (Class::ZWJ, _) => Break::Prohibited,

            // LB9
            (ref x, Class::CM) | (ref x, Class::ZWJ)
                if *x != Class::BK
                    && *x != Class::CR
                    && *x != Class::LF
                    && *x != Class::NL
                    && *x != Class::SP
                    && *x != Class::ZW =>
            {
                self.treat_next_n1_as = Some(n1);
                Break::Prohibited
            }

            // LB11
            (_, Class::WJ) | (Class::WJ, _) => Break::Prohibited,

            // LB12
            (Class::GL, _) => Break::Prohibited,

            // LB12a
            (ref x, Class::GL) if *x != Class::SP && *x != Class::BA && *x != Class::HY => {
                Break::Prohibited
            }

            // LB13
            (_, Class::CL) | (_, Class::CP) | (_, Class::EX) | (_, Class::IS) | (_, Class::SY) => {
                Break::Prohibited
            }

            // LB14
            (Class::OP, _) => Break::Prohibited,
            (Class::SP, _) if self.class_before_spaces == Some(Class::OP) => Break::Prohibited,

            // LB15
            (Class::QU, Class::OP) => Break::Prohibited,
            (Class::SP, Class::OP) if self.class_before_spaces == Some(Class::QU) => {
                Break::Prohibited
            }

            // LB16
            (Class::CL, Class::NS) | (Class::CP, Class::NS) => Break::Prohibited,
            (Class::SP, Class::NS)
                if (self.class_before_spaces == Some(Class::CL))
                    | (self.class_before_spaces == Some(Class::CP)) =>
            {
                Break::Prohibited
            }

            // LB17
            (Class::B2, Class::B2) => Break::Prohibited,
            (Class::SP, Class::B2) if self.class_before_spaces == Some(Class::B2) => {
                Break::Prohibited
            }

            // LB18
            (Class::SP, _) => Break::Opportunity,

            // LB19
            (Class::QU, _) | (_, Class::QU) => Break::Prohibited,

            // LB20
            (Class::CB, _) | (_, Class::CB) => Break::Opportunity,

            // LB21
            (x, Class::BA) | (x, Class::HY) => {
                // LB21a
                if x == Class::HL {
                    self.next_is_prohibited = true;
                }
                Break::Prohibited
            }
            (_, Class::NS) | (Class::BB, _) => Break::Prohibited,

            // LB21b
            (Class::SY, Class::HL) => Break::Prohibited,

            // LB22
            (Class::AL, Class::IN)
            | (Class::HL, Class::IN)
            | (Class::EX, Class::IN)
            | (Class::ID, Class::IN)
            | (Class::EB, Class::IN)
            | (Class::EM, Class::IN)
            | (Class::IN, Class::IN)
            | (Class::NU, Class::IN) => Break::Prohibited,

            // LB23
            (Class::AL, Class::NU)
            | (Class::HL, Class::NU)
            | (Class::NU, Class::AL)
            | (Class::NU, Class::HL) => Break::Prohibited,

            // LB23a
            (Class::PR, Class::ID)
            | (Class::PR, Class::EB)
            | (Class::PR, Class::EM)
            | (Class::ID, Class::PO)
            | (Class::EB, Class::PO)
            | (Class::EM, Class::PO) => Break::Prohibited,

            // LB24 - regexp thing not implemented
            (Class::PR, Class::AL)
            | (Class::PR, Class::HL)
            | (Class::PO, Class::AL)
            | (Class::PO, Class::HL)
            | (Class::AL, Class::PR)
            | (Class::AL, Class::PO)
            | (Class::HL, Class::PR)
            | (Class::HL, Class::PO) => Break::Prohibited,

            //LB25
            (Class::CL, Class::PO)
            | (Class::CP, Class::PO)
            | (Class::CL, Class::PR)
            | (Class::CP, Class::PR)
            | (Class::NU, Class::PO)
            | (Class::NU, Class::PR)
            | (Class::PO, Class::OP)
            | (Class::PO, Class::NU)
            | (Class::PR, Class::OP)
            | (Class::PR, Class::NU)
            | (Class::HY, Class::NU)
            | (Class::IS, Class::NU)
            | (Class::NU, Class::NU)
            | (Class::SY, Class::NU) => Break::Prohibited,

            // LB26
            (Class::JL, Class::JL)
            | (Class::JL, Class::JV)
            | (Class::JL, Class::H2)
            | (Class::JL, Class::H3)
            | (Class::JV, Class::JV)
            | (Class::JV, Class::JT)
            | (Class::H2, Class::JV)
            | (Class::H2, Class::JT)
            | (Class::JT, Class::JT)
            | (Class::H3, Class::JT) => Break::Prohibited,

            // LB27
            (Class::JL, Class::IN)
            | (Class::JV, Class::IN)
            | (Class::JT, Class::IN)
            | (Class::H2, Class::IN)
            | (Class::H3, Class::IN)
            | (Class::JL, Class::PO)
            | (Class::JV, Class::PO)
            | (Class::JT, Class::PO)
            | (Class::H2, Class::PO)
            | (Class::H3, Class::PO)
            | (Class::PR, Class::JL)
            | (Class::PR, Class::JV)
            | (Class::PR, Class::JT)
            | (Class::PR, Class::H2)
            | (Class::PR, Class::H3) => Break::Prohibited,

            // LB28
            (Class::AL, Class::AL)
            | (Class::HL, Class::AL)
            | (Class::AL, Class::HL)
            | (Class::HL, Class::HL) => Break::Prohibited,

            // LB29
            (Class::IS, Class::AL) | (Class::IS, Class::HL) => Break::Prohibited,

            // LB30
            (Class::AL, Class::OP)
            | (Class::HL, Class::OP)
            | (Class::NU, Class::OP)
            | (Class::CP, Class::AL)
            | (Class::CP, Class::HL)
            | (Class::CP, Class::NU) => Break::Prohibited,

            // LB30a
            (Class::RI, Class::RI) if self.ri_count % 2 == 0 => Break::Prohibited,

            // LB30b
            (Class::EB, Class::EM) => Break::Prohibited,

            // LB31
            (_, _) => Break::Opportunity,
        };

        // LB8, LB14, LB15, LB16, LB17
        if n1 == Class::SP && n2 != Class::SP {
            self.class_before_spaces = None;
        }
        b
    }
}
/// Provide information as to whether a line break can be appended for each
/// `char` in the input.
impl<'a> Iterator for BreakInfo<'a> {
    type Item = (char, Break);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let tuple = match (self.iter.next(), self.iter.peek()) {
            (Some(a), Some(&b)) => (
                a,
                self.get_break(convert_to_break_class(a), convert_to_break_class(b)),
            ),
            (None, Some(_)) => unreachable!(),
            (Some(a), None) => (a, Break::Opportunity),
            (None, None) => {
                return None;
            }
        };
        Some(tuple)
    }
}