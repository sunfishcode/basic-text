//! On input, several disallowed scalar values are replaced, so that content
//! containing them can still be read, but applications don't have to
//! handle them.

use crate::unicode::{BOM, FF, LS, NEL, ORC, PS, REPL, WJ};
use std::collections::VecDeque;

#[inline]
pub fn replace(c: char, queue: &mut VecDeque<char>) {
    match c {
        BOM => queue.push_back(WJ),
        '\u{149}' => {
            queue.push_back('\u{2bc}');
            queue.push_back('\u{6e}');
        }
        '\u{673}' => {
            queue.push_back('\u{627}');
            queue.push_back('\u{65f}');
        }
        '\u{f77}' => {
            queue.push_back('\u{fb2}');
            queue.push_back('\u{f81}');
        }
        '\u{f79}' => {
            queue.push_back('\u{fb3}');
            queue.push_back('\u{f81}');
        }
        '\u{17a3}' => queue.push_back('\u{17a2}'),
        '\u{17a4}' => {
            queue.push_back('\u{17a2}');
            queue.push_back('\u{17b6}');
        }
        // Discouraged characters
        '\u{2df5}' => {
            queue.push_back('\u{2ded}');
            queue.push_back('\u{2dee}');
        }
        '\u{111c4}' => {
            queue.push_back('\u{1118f}');
            queue.push_back('\u{11180}');
        }
        LS | PS => queue.push_back(' '),
        // Latin Ligatures
        '\u{fb00}' => {
            queue.push_back('f');
            queue.push_back('f');
        }
        '\u{fb01}' => {
            queue.push_back('f');
            queue.push_back('i');
        }
        '\u{fb02}' => {
            queue.push_back('f');
            queue.push_back('l');
        }
        '\u{fb03}' => {
            queue.push_back('f');
            queue.push_back('f');
            queue.push_back('i');
        }
        '\u{fb04}' => {
            queue.push_back('f');
            queue.push_back('f');
            queue.push_back('l');
        }
        '\u{fb05}' => {
            queue.push_back('ſ');
            queue.push_back('t');
        }
        '\u{fb06}' => {
            queue.push_back('s');
            queue.push_back('t');
        }
        FF | NEL => queue.push_back(' '),
        // Control codes: C0 (except '\n', '\t', FF, and ESC), DEL, C1 (except NEL)
        '\u{0}' | '\u{1}' | '\u{2}' | '\u{3}' |
        '\u{4}' | '\u{5}' | '\u{6}' | '\u{7}' |
        '\u{8}' | '\u{b}' |
        '\u{d}' | '\u{e}' | '\u{f}' |
        '\u{10}' | '\u{11}' | '\u{12}' | '\u{13}' |
        '\u{14}' | '\u{15}' | '\u{16}' | '\u{17}' |
        '\u{18}' | '\u{19}' | '\u{1a}' |
        '\u{1c}' | '\u{1d}' | '\u{1e}' | '\u{1f}' |
        '\u{7f}' |
        '\u{80}' | '\u{81}' | '\u{82}' | '\u{83}' |
        '\u{84}' | '\u{86}' | '\u{87}' |
        '\u{88}' | '\u{89}' | '\u{8a}' | '\u{8b}' |
        '\u{8c}' | '\u{8d}' | '\u{8e}' | '\u{8f}' |
        '\u{90}' | '\u{91}' | '\u{92}' | '\u{93}' |
        '\u{94}' | '\u{95}' | '\u{96}' | '\u{97}' |
        '\u{98}' | '\u{99}' | '\u{9a}' | '\u{9b}' |
        '\u{9c}' | '\u{9d}' | '\u{9e}' | '\u{9f}' |
        // Angle brackets
        '\u{2329}' | '\u{232a}' |
        // Interlinear Annotations
        '\u{fff9}'..='\u{fffb}' |
        // Unassigned characters with replacements.
        '\u{9e4}' | '\u{9e5}' | '\u{a64}' | '\u{a65}' |
        '\u{ae4}' | '\u{ae5}' | '\u{b64}' | '\u{b65}' |
        '\u{be4}' | '\u{be5}' | '\u{c64}' | '\u{c65}' |
        '\u{ce4}' | '\u{ce5}' | '\u{d64}' | '\u{d65}' |
        '\u{2072}' | '\u{2073}' |
        '\u{1d455}' | '\u{1d49d}' | '\u{1d4a0}' | '\u{1d4a1}' |
        '\u{1d4a3}' | '\u{1d4a4}' | '\u{1d4a7}' | '\u{1d4a8}' |
        '\u{1d4ad}' | '\u{1d4ba}' | '\u{1d4bc}' | '\u{1d4c4}' |
        '\u{1d506}' | '\u{1d50b}' | '\u{1d50c}' | '\u{1d515}' |
        '\u{1d51d}' | '\u{1d53a}' | '\u{1d53f}' | '\u{1d545}' |
        '\u{1d547}' | '\u{1d548}' | '\u{1d549}' | '\u{1d551}' |
        // Object Replacement Character
        ORC |
        // Khmer characters erroneously invented by Unicode.
        '\u{17b4}' | '\u{17b5}' | '\u{17d8}' |
        // Deprecated Format Characters
        '\u{206a}'..='\u{206f}' |
        // Bidirectional Format Characters
        '\u{202a}' | '\u{202b}' | '\u{202c}' | '\u{202d}' | '\u{202e}' |
        '\u{2066}' | '\u{2067}' | '\u{2068}' | '\u{2069}' |
        // Tag Characters
        '\u{e0000}'..='\u{e007f}' |
        // Noncharacters
        '\u{fffe}' ..= '\u{ffff}' |
        '\u{1fffe}' ..= '\u{1ffff}' |
        '\u{2fffe}' ..= '\u{2ffff}' |
        '\u{3fffe}' ..= '\u{3ffff}' |
        '\u{4fffe}' ..= '\u{4ffff}' |
        '\u{5fffe}' ..= '\u{5ffff}' |
        '\u{6fffe}' ..= '\u{6ffff}' |
        '\u{7fffe}' ..= '\u{7ffff}' |
        '\u{8fffe}' ..= '\u{8ffff}' |
        '\u{9fffe}' ..= '\u{9ffff}' |
        '\u{afffe}' ..= '\u{affff}' |
        '\u{bfffe}' ..= '\u{bffff}' |
        '\u{cfffe}' ..= '\u{cffff}' |
        '\u{dfffe}' ..= '\u{dffff}' |
        '\u{efffe}' ..= '\u{effff}' |
        '\u{ffffe}' ..= '\u{fffff}' |
        '\u{10fffe}' ..= '\u{10ffff}' |
        '\u{fdd0}'..='\u{fdef}' => queue.push_back(REPL),

        c => queue.push_back(c),
    }
}
