//! On output, several disallowed scalar values are rejected, to catch
//! applications attempting to use them.

use crate::{
    replace,
    unicode::{BOM, ESC, ORC},
};
use std::collections::VecDeque;
use thiserror::Error;

/// Test whether the given Unicode scalar value is valid in a Basic Text string.
#[inline]
pub fn check_basic_text_char(c: char) -> Result<(), BasicTextError> {
    match c {
        // Newline and tab are allowed, and escape is handled specially.
        c if c.is_control() && c != '\n' && c != '\t' && c != ESC => control(c),
        c @ '\u{149}'
        | c @ '\u{673}'
        | c @ '\u{f77}'
        | c @ '\u{f79}'
        | c @ '\u{17a3}'
        | c @ '\u{17a4}'
        | c @ '\u{2329}'
        | c @ '\u{232a}'
        | c @ '\u{2126}'
        | c @ '\u{212a}'
        | c @ '\u{212b}'
        | c @ '\u{2df5}'
        | c @ '\u{111c4}'
        | c @ '\u{fb00}'..='\u{fb06}'
        | c @ '\u{9e4}'
        | c @ '\u{9e5}'
        | c @ '\u{a64}'
        | c @ '\u{a65}'
        | c @ '\u{ae4}'
        | c @ '\u{ae5}'
        | c @ '\u{b64}'
        | c @ '\u{b65}'
        | c @ '\u{be4}'
        | c @ '\u{be5}'
        | c @ '\u{c64}'
        | c @ '\u{c65}'
        | c @ '\u{ce4}'
        | c @ '\u{ce5}'
        | c @ '\u{d64}'
        | c @ '\u{d65}'
        | c @ '\u{2072}'
        | c @ '\u{2073}'
        | c @ '\u{1d455}'
        | c @ '\u{1d49d}'
        | c @ '\u{1d4a0}'
        | c @ '\u{1d4a1}'
        | c @ '\u{1d4a3}'
        | c @ '\u{1d4a4}'
        | c @ '\u{1d4a7}'
        | c @ '\u{1d4a8}'
        | c @ '\u{1d4ad}'
        | c @ '\u{1d4ba}'
        | c @ '\u{1d4bc}'
        | c @ '\u{1d4c4}'
        | c @ '\u{1d506}'
        | c @ '\u{1d50b}'
        | c @ '\u{1d50c}'
        | c @ '\u{1d515}'
        | c @ '\u{1d51d}'
        | c @ '\u{1d53a}'
        | c @ '\u{1d53f}'
        | c @ '\u{1d545}'
        | c @ '\u{1d547}'
        | c @ '\u{1d548}'
        | c @ '\u{1d549}'
        | c @ '\u{1d551}' => replacement(c),
        '\u{e0001}' => language_tag(),
        '\u{fff9}'..='\u{fffb}' => interlinear_annotation(),
        c @ '\u{17b4}' | c @ '\u{17b5}' | c @ '\u{17d8}' => discouraged(c),
        c @ '\u{206a}'..='\u{206f}' => deprecated_format_character(c),
        '\u{2028}' => line_separation(),
        '\u{2029}' => para_separation(),
        '\u{202a}' | '\u{202b}' | '\u{202c}' | '\u{202d}' | '\u{202e}' | '\u{2066}'
        | '\u{2067}' | '\u{2068}' | '\u{2069}' => bidirectional_formatting_character(),
        '\u{fffe}'..='\u{ffff}'
        | '\u{1fffe}'..='\u{1ffff}'
        | '\u{2fffe}'..='\u{2ffff}'
        | '\u{3fffe}'..='\u{3ffff}'
        | '\u{4fffe}'..='\u{4ffff}'
        | '\u{5fffe}'..='\u{5ffff}'
        | '\u{6fffe}'..='\u{6ffff}'
        | '\u{7fffe}'..='\u{7ffff}'
        | '\u{8fffe}'..='\u{8ffff}'
        | '\u{9fffe}'..='\u{9ffff}'
        | '\u{afffe}'..='\u{affff}'
        | '\u{bfffe}'..='\u{bffff}'
        | '\u{cfffe}'..='\u{cffff}'
        | '\u{dfffe}'..='\u{dffff}'
        | '\u{efffe}'..='\u{effff}'
        | '\u{ffffe}'..='\u{fffff}'
        | '\u{10fffe}'..='\u{10ffff}'
        | '\u{fdd0}'..='\u{fdef}' => noncharacter(),
        '\u{f900}' | '\u{f901}' | '\u{f902}' | '\u{f903}' | '\u{f904}' | '\u{f905}'
        | '\u{f906}' | '\u{f907}' | '\u{f908}' | '\u{f909}' | '\u{f90a}' | '\u{f90b}'
        | '\u{f90c}' | '\u{f90d}' | '\u{f90e}' | '\u{f90f}' | '\u{f910}' | '\u{f911}'
        | '\u{f912}' | '\u{f913}' | '\u{f914}' | '\u{f915}' | '\u{f916}' | '\u{f917}'
        | '\u{f918}' | '\u{f919}' | '\u{f91a}' | '\u{f91b}' | '\u{f91c}' | '\u{f91d}'
        | '\u{f91e}' | '\u{f91f}' | '\u{f920}' | '\u{f921}' | '\u{f922}' | '\u{f923}'
        | '\u{f924}' | '\u{f925}' | '\u{f926}' | '\u{f927}' | '\u{f928}' | '\u{f929}'
        | '\u{f92a}' | '\u{f92b}' | '\u{f92c}' | '\u{f92d}' | '\u{f92e}' | '\u{f92f}'
        | '\u{f930}' | '\u{f931}' | '\u{f932}' | '\u{f933}' | '\u{f934}' | '\u{f935}'
        | '\u{f936}' | '\u{f937}' | '\u{f938}' | '\u{f939}' | '\u{f93a}' | '\u{f93b}'
        | '\u{f93c}' | '\u{f93d}' | '\u{f93e}' | '\u{f93f}' | '\u{f940}' | '\u{f941}'
        | '\u{f942}' | '\u{f943}' | '\u{f944}' | '\u{f945}' | '\u{f946}' | '\u{f947}'
        | '\u{f948}' | '\u{f949}' | '\u{f94a}' | '\u{f94b}' | '\u{f94c}' | '\u{f94d}'
        | '\u{f94e}' | '\u{f94f}' | '\u{f950}' | '\u{f951}' | '\u{f952}' | '\u{f953}'
        | '\u{f954}' | '\u{f955}' | '\u{f956}' | '\u{f957}' | '\u{f958}' | '\u{f959}'
        | '\u{f95a}' | '\u{f95b}' | '\u{f95c}' | '\u{f95d}' | '\u{f95e}' | '\u{f95f}'
        | '\u{f960}' | '\u{f961}' | '\u{f962}' | '\u{f963}' | '\u{f964}' | '\u{f965}'
        | '\u{f966}' | '\u{f967}' | '\u{f968}' | '\u{f969}' | '\u{f96a}' | '\u{f96b}'
        | '\u{f96c}' | '\u{f96d}' | '\u{f96e}' | '\u{f96f}' | '\u{f970}' | '\u{f971}'
        | '\u{f972}' | '\u{f973}' | '\u{f974}' | '\u{f975}' | '\u{f976}' | '\u{f977}'
        | '\u{f978}' | '\u{f979}' | '\u{f97a}' | '\u{f97b}' | '\u{f97c}' | '\u{f97d}'
        | '\u{f97e}' | '\u{f97f}' | '\u{f980}' | '\u{f981}' | '\u{f982}' | '\u{f983}'
        | '\u{f984}' | '\u{f985}' | '\u{f986}' | '\u{f987}' | '\u{f988}' | '\u{f989}'
        | '\u{f98a}' | '\u{f98b}' | '\u{f98c}' | '\u{f98d}' | '\u{f98e}' | '\u{f98f}'
        | '\u{f990}' | '\u{f991}' | '\u{f992}' | '\u{f993}' | '\u{f994}' | '\u{f995}'
        | '\u{f996}' | '\u{f997}' | '\u{f998}' | '\u{f999}' | '\u{f99a}' | '\u{f99b}'
        | '\u{f99c}' | '\u{f99d}' | '\u{f99e}' | '\u{f99f}' | '\u{f9a0}' | '\u{f9a1}'
        | '\u{f9a2}' | '\u{f9a3}' | '\u{f9a4}' | '\u{f9a5}' | '\u{f9a6}' | '\u{f9a7}'
        | '\u{f9a8}' | '\u{f9a9}' | '\u{f9aa}' | '\u{f9ab}' | '\u{f9ac}' | '\u{f9ad}'
        | '\u{f9ae}' | '\u{f9af}' | '\u{f9b0}' | '\u{f9b1}' | '\u{f9b2}' | '\u{f9b3}'
        | '\u{f9b4}' | '\u{f9b5}' | '\u{f9b6}' | '\u{f9b7}' | '\u{f9b8}' | '\u{f9b9}'
        | '\u{f9ba}' | '\u{f9bb}' | '\u{f9bc}' | '\u{f9bd}' | '\u{f9be}' | '\u{f9bf}'
        | '\u{f9c0}' | '\u{f9c1}' | '\u{f9c2}' | '\u{f9c3}' | '\u{f9c4}' | '\u{f9c5}'
        | '\u{f9c6}' | '\u{f9c7}' | '\u{f9c8}' | '\u{f9c9}' | '\u{f9ca}' | '\u{f9cb}'
        | '\u{f9cc}' | '\u{f9cd}' | '\u{f9ce}' | '\u{f9cf}' | '\u{f9d0}' | '\u{f9d1}'
        | '\u{f9d2}' | '\u{f9d3}' | '\u{f9d4}' | '\u{f9d5}' | '\u{f9d6}' | '\u{f9d7}'
        | '\u{f9d8}' | '\u{f9d9}' | '\u{f9da}' | '\u{f9db}' | '\u{f9dc}' | '\u{f9dd}'
        | '\u{f9de}' | '\u{f9df}' | '\u{f9e0}' | '\u{f9e1}' | '\u{f9e2}' | '\u{f9e3}'
        | '\u{f9e4}' | '\u{f9e5}' | '\u{f9e6}' | '\u{f9e7}' | '\u{f9e8}' | '\u{f9e9}'
        | '\u{f9ea}' | '\u{f9eb}' | '\u{f9ec}' | '\u{f9ed}' | '\u{f9ee}' | '\u{f9ef}'
        | '\u{f9f0}' | '\u{f9f1}' | '\u{f9f2}' | '\u{f9f3}' | '\u{f9f4}' | '\u{f9f5}'
        | '\u{f9f6}' | '\u{f9f7}' | '\u{f9f8}' | '\u{f9f9}' | '\u{f9fa}' | '\u{f9fb}'
        | '\u{f9fc}' | '\u{f9fd}' | '\u{f9fe}' | '\u{f9ff}' | '\u{fa00}' | '\u{fa01}'
        | '\u{fa02}' | '\u{fa03}' | '\u{fa04}' | '\u{fa05}' | '\u{fa06}' | '\u{fa07}'
        | '\u{fa08}' | '\u{fa09}' | '\u{fa0a}' | '\u{fa0b}' | '\u{fa0c}' | '\u{fa0d}'
        | '\u{fa10}' | '\u{fa12}' | '\u{fa15}' | '\u{fa16}' | '\u{fa17}' | '\u{fa18}'
        | '\u{fa19}' | '\u{fa1a}' | '\u{fa1b}' | '\u{fa1c}' | '\u{fa1d}' | '\u{fa1e}'
        | '\u{fa20}' | '\u{fa22}' | '\u{fa25}' | '\u{fa26}' | '\u{fa2a}' | '\u{fa2b}'
        | '\u{fa2c}' | '\u{fa2d}' | '\u{fa2e}' | '\u{fa2f}' | '\u{fa30}' | '\u{fa31}'
        | '\u{fa32}' | '\u{fa33}' | '\u{fa34}' | '\u{fa35}' | '\u{fa36}' | '\u{fa37}'
        | '\u{fa38}' | '\u{fa39}' | '\u{fa3a}' | '\u{fa3b}' | '\u{fa3c}' | '\u{fa3d}'
        | '\u{fa3e}' | '\u{fa3f}' | '\u{fa40}' | '\u{fa41}' | '\u{fa42}' | '\u{fa43}'
        | '\u{fa44}' | '\u{fa45}' | '\u{fa46}' | '\u{fa47}' | '\u{fa48}' | '\u{fa49}'
        | '\u{fa4a}' | '\u{fa4b}' | '\u{fa4c}' | '\u{fa4d}' | '\u{fa4e}' | '\u{fa4f}'
        | '\u{fa50}' | '\u{fa51}' | '\u{fa52}' | '\u{fa53}' | '\u{fa54}' | '\u{fa55}'
        | '\u{fa56}' | '\u{fa57}' | '\u{fa58}' | '\u{fa59}' | '\u{fa5a}' | '\u{fa5b}'
        | '\u{fa5c}' | '\u{fa5d}' | '\u{fa5e}' | '\u{fa5f}' | '\u{fa60}' | '\u{fa61}'
        | '\u{fa62}' | '\u{fa63}' | '\u{fa64}' | '\u{fa65}' | '\u{fa66}' | '\u{fa67}'
        | '\u{fa68}' | '\u{fa69}' | '\u{fa6a}' | '\u{fa6b}' | '\u{fa6c}' | '\u{fa6d}'
        | '\u{fa70}' | '\u{fa71}' | '\u{fa72}' | '\u{fa73}' | '\u{fa74}' | '\u{fa75}'
        | '\u{fa76}' | '\u{fa77}' | '\u{fa78}' | '\u{fa79}' | '\u{fa7a}' | '\u{fa7b}'
        | '\u{fa7c}' | '\u{fa7d}' | '\u{fa7e}' | '\u{fa7f}' | '\u{fa80}' | '\u{fa81}'
        | '\u{fa82}' | '\u{fa83}' | '\u{fa84}' | '\u{fa85}' | '\u{fa86}' | '\u{fa87}'
        | '\u{fa88}' | '\u{fa89}' | '\u{fa8a}' | '\u{fa8b}' | '\u{fa8c}' | '\u{fa8d}'
        | '\u{fa8e}' | '\u{fa8f}' | '\u{fa90}' | '\u{fa91}' | '\u{fa92}' | '\u{fa93}'
        | '\u{fa94}' | '\u{fa95}' | '\u{fa96}' | '\u{fa97}' | '\u{fa98}' | '\u{fa99}'
        | '\u{fa9a}' | '\u{fa9b}' | '\u{fa9c}' | '\u{fa9d}' | '\u{fa9e}' | '\u{fa9f}'
        | '\u{faa0}' | '\u{faa1}' | '\u{faa2}' | '\u{faa3}' | '\u{faa4}' | '\u{faa5}'
        | '\u{faa6}' | '\u{faa7}' | '\u{faa8}' | '\u{faa9}' | '\u{faaa}' | '\u{faab}'
        | '\u{faac}' | '\u{faad}' | '\u{faae}' | '\u{faaf}' | '\u{fab0}' | '\u{fab1}'
        | '\u{fab2}' | '\u{fab3}' | '\u{fab4}' | '\u{fab5}' | '\u{fab6}' | '\u{fab7}'
        | '\u{fab8}' | '\u{fab9}' | '\u{faba}' | '\u{fabb}' | '\u{fabc}' | '\u{fabd}'
        | '\u{fabe}' | '\u{fabf}' | '\u{fac0}' | '\u{fac1}' | '\u{fac2}' | '\u{fac3}'
        | '\u{fac4}' | '\u{fac5}' | '\u{fac6}' | '\u{fac7}' | '\u{fac8}' | '\u{fac9}'
        | '\u{faca}' | '\u{facb}' | '\u{facc}' | '\u{facd}' | '\u{face}' | '\u{facf}'
        | '\u{fad0}' | '\u{fad1}' | '\u{fad2}' | '\u{fad3}' | '\u{fad4}' | '\u{fad5}'
        | '\u{fad6}' | '\u{fad7}' | '\u{fad8}' | '\u{fad9}' | '\u{2f800}' | '\u{2f801}'
        | '\u{2f802}' | '\u{2f803}' | '\u{2f804}' | '\u{2f805}' | '\u{2f806}' | '\u{2f807}'
        | '\u{2f808}' | '\u{2f809}' | '\u{2f80a}' | '\u{2f80b}' | '\u{2f80c}' | '\u{2f80d}'
        | '\u{2f80e}' | '\u{2f80f}' | '\u{2f810}' | '\u{2f811}' | '\u{2f812}' | '\u{2f813}'
        | '\u{2f814}' | '\u{2f815}' | '\u{2f816}' | '\u{2f817}' | '\u{2f818}' | '\u{2f819}'
        | '\u{2f81a}' | '\u{2f81b}' | '\u{2f81c}' | '\u{2f81d}' | '\u{2f81e}' | '\u{2f81f}'
        | '\u{2f820}' | '\u{2f821}' | '\u{2f822}' | '\u{2f823}' | '\u{2f824}' | '\u{2f825}'
        | '\u{2f826}' | '\u{2f827}' | '\u{2f828}' | '\u{2f829}' | '\u{2f82a}' | '\u{2f82b}'
        | '\u{2f82c}' | '\u{2f82d}' | '\u{2f82e}' | '\u{2f82f}' | '\u{2f830}' | '\u{2f831}'
        | '\u{2f832}' | '\u{2f833}' | '\u{2f834}' | '\u{2f835}' | '\u{2f836}' | '\u{2f837}'
        | '\u{2f838}' | '\u{2f839}' | '\u{2f83a}' | '\u{2f83b}' | '\u{2f83c}' | '\u{2f83d}'
        | '\u{2f83e}' | '\u{2f83f}' | '\u{2f840}' | '\u{2f841}' | '\u{2f842}' | '\u{2f843}'
        | '\u{2f844}' | '\u{2f845}' | '\u{2f846}' | '\u{2f847}' | '\u{2f848}' | '\u{2f849}'
        | '\u{2f84a}' | '\u{2f84b}' | '\u{2f84c}' | '\u{2f84d}' | '\u{2f84e}' | '\u{2f84f}'
        | '\u{2f850}' | '\u{2f851}' | '\u{2f852}' | '\u{2f853}' | '\u{2f854}' | '\u{2f855}'
        | '\u{2f856}' | '\u{2f857}' | '\u{2f858}' | '\u{2f859}' | '\u{2f85a}' | '\u{2f85b}'
        | '\u{2f85c}' | '\u{2f85d}' | '\u{2f85e}' | '\u{2f85f}' | '\u{2f860}' | '\u{2f861}'
        | '\u{2f862}' | '\u{2f863}' | '\u{2f864}' | '\u{2f865}' | '\u{2f866}' | '\u{2f867}'
        | '\u{2f868}' | '\u{2f869}' | '\u{2f86a}' | '\u{2f86b}' | '\u{2f86c}' | '\u{2f86d}'
        | '\u{2f86e}' | '\u{2f86f}' | '\u{2f870}' | '\u{2f871}' | '\u{2f872}' | '\u{2f873}'
        | '\u{2f874}' | '\u{2f875}' | '\u{2f876}' | '\u{2f877}' | '\u{2f878}' | '\u{2f879}'
        | '\u{2f87a}' | '\u{2f87b}' | '\u{2f87c}' | '\u{2f87d}' | '\u{2f87e}' | '\u{2f87f}'
        | '\u{2f880}' | '\u{2f881}' | '\u{2f882}' | '\u{2f883}' | '\u{2f884}' | '\u{2f885}'
        | '\u{2f886}' | '\u{2f887}' | '\u{2f888}' | '\u{2f889}' | '\u{2f88a}' | '\u{2f88b}'
        | '\u{2f88c}' | '\u{2f88d}' | '\u{2f88e}' | '\u{2f88f}' | '\u{2f890}' | '\u{2f891}'
        | '\u{2f892}' | '\u{2f893}' | '\u{2f894}' | '\u{2f895}' | '\u{2f896}' | '\u{2f897}'
        | '\u{2f898}' | '\u{2f899}' | '\u{2f89a}' | '\u{2f89b}' | '\u{2f89c}' | '\u{2f89d}'
        | '\u{2f89e}' | '\u{2f89f}' | '\u{2f8a0}' | '\u{2f8a1}' | '\u{2f8a2}' | '\u{2f8a3}'
        | '\u{2f8a4}' | '\u{2f8a5}' | '\u{2f8a6}' | '\u{2f8a7}' | '\u{2f8a8}' | '\u{2f8a9}'
        | '\u{2f8aa}' | '\u{2f8ab}' | '\u{2f8ac}' | '\u{2f8ad}' | '\u{2f8ae}' | '\u{2f8af}'
        | '\u{2f8b0}' | '\u{2f8b1}' | '\u{2f8b2}' | '\u{2f8b3}' | '\u{2f8b4}' | '\u{2f8b5}'
        | '\u{2f8b6}' | '\u{2f8b7}' | '\u{2f8b8}' | '\u{2f8b9}' | '\u{2f8ba}' | '\u{2f8bb}'
        | '\u{2f8bc}' | '\u{2f8bd}' | '\u{2f8be}' | '\u{2f8bf}' | '\u{2f8c0}' | '\u{2f8c1}'
        | '\u{2f8c2}' | '\u{2f8c3}' | '\u{2f8c4}' | '\u{2f8c5}' | '\u{2f8c6}' | '\u{2f8c7}'
        | '\u{2f8c8}' | '\u{2f8c9}' | '\u{2f8ca}' | '\u{2f8cb}' | '\u{2f8cc}' | '\u{2f8cd}'
        | '\u{2f8ce}' | '\u{2f8cf}' | '\u{2f8d0}' | '\u{2f8d1}' | '\u{2f8d2}' | '\u{2f8d3}'
        | '\u{2f8d4}' | '\u{2f8d5}' | '\u{2f8d6}' | '\u{2f8d7}' | '\u{2f8d8}' | '\u{2f8d9}'
        | '\u{2f8da}' | '\u{2f8db}' | '\u{2f8dc}' | '\u{2f8dd}' | '\u{2f8de}' | '\u{2f8df}'
        | '\u{2f8e0}' | '\u{2f8e1}' | '\u{2f8e2}' | '\u{2f8e3}' | '\u{2f8e4}' | '\u{2f8e5}'
        | '\u{2f8e6}' | '\u{2f8e7}' | '\u{2f8e8}' | '\u{2f8e9}' | '\u{2f8ea}' | '\u{2f8eb}'
        | '\u{2f8ec}' | '\u{2f8ed}' | '\u{2f8ee}' | '\u{2f8ef}' | '\u{2f8f0}' | '\u{2f8f1}'
        | '\u{2f8f2}' | '\u{2f8f3}' | '\u{2f8f4}' | '\u{2f8f5}' | '\u{2f8f6}' | '\u{2f8f7}'
        | '\u{2f8f8}' | '\u{2f8f9}' | '\u{2f8fa}' | '\u{2f8fb}' | '\u{2f8fc}' | '\u{2f8fd}'
        | '\u{2f8fe}' | '\u{2f8ff}' | '\u{2f900}' | '\u{2f901}' | '\u{2f902}' | '\u{2f903}'
        | '\u{2f904}' | '\u{2f905}' | '\u{2f906}' | '\u{2f907}' | '\u{2f908}' | '\u{2f909}'
        | '\u{2f90a}' | '\u{2f90b}' | '\u{2f90c}' | '\u{2f90d}' | '\u{2f90e}' | '\u{2f90f}'
        | '\u{2f910}' | '\u{2f911}' | '\u{2f912}' | '\u{2f913}' | '\u{2f914}' | '\u{2f915}'
        | '\u{2f916}' | '\u{2f917}' | '\u{2f918}' | '\u{2f919}' | '\u{2f91a}' | '\u{2f91b}'
        | '\u{2f91c}' | '\u{2f91d}' | '\u{2f91e}' | '\u{2f91f}' | '\u{2f920}' | '\u{2f921}'
        | '\u{2f922}' | '\u{2f923}' | '\u{2f924}' | '\u{2f925}' | '\u{2f926}' | '\u{2f927}'
        | '\u{2f928}' | '\u{2f929}' | '\u{2f92a}' | '\u{2f92b}' | '\u{2f92c}' | '\u{2f92d}'
        | '\u{2f92e}' | '\u{2f92f}' | '\u{2f930}' | '\u{2f931}' | '\u{2f932}' | '\u{2f933}'
        | '\u{2f934}' | '\u{2f935}' | '\u{2f936}' | '\u{2f937}' | '\u{2f938}' | '\u{2f939}'
        | '\u{2f93a}' | '\u{2f93b}' | '\u{2f93c}' | '\u{2f93d}' | '\u{2f93e}' | '\u{2f93f}'
        | '\u{2f940}' | '\u{2f941}' | '\u{2f942}' | '\u{2f943}' | '\u{2f944}' | '\u{2f945}'
        | '\u{2f946}' | '\u{2f947}' | '\u{2f948}' | '\u{2f949}' | '\u{2f94a}' | '\u{2f94b}'
        | '\u{2f94c}' | '\u{2f94d}' | '\u{2f94e}' | '\u{2f94f}' | '\u{2f950}' | '\u{2f951}'
        | '\u{2f952}' | '\u{2f953}' | '\u{2f954}' | '\u{2f955}' | '\u{2f956}' | '\u{2f957}'
        | '\u{2f958}' | '\u{2f959}' | '\u{2f95a}' | '\u{2f95b}' | '\u{2f95c}' | '\u{2f95d}'
        | '\u{2f95e}' | '\u{2f95f}' | '\u{2f960}' | '\u{2f961}' | '\u{2f962}' | '\u{2f963}'
        | '\u{2f964}' | '\u{2f965}' | '\u{2f966}' | '\u{2f967}' | '\u{2f968}' | '\u{2f969}'
        | '\u{2f96a}' | '\u{2f96b}' | '\u{2f96c}' | '\u{2f96d}' | '\u{2f96e}' | '\u{2f96f}'
        | '\u{2f970}' | '\u{2f971}' | '\u{2f972}' | '\u{2f973}' | '\u{2f974}' | '\u{2f975}'
        | '\u{2f976}' | '\u{2f977}' | '\u{2f978}' | '\u{2f979}' | '\u{2f97a}' | '\u{2f97b}'
        | '\u{2f97c}' | '\u{2f97d}' | '\u{2f97e}' | '\u{2f97f}' | '\u{2f980}' | '\u{2f981}'
        | '\u{2f982}' | '\u{2f983}' | '\u{2f984}' | '\u{2f985}' | '\u{2f986}' | '\u{2f987}'
        | '\u{2f988}' | '\u{2f989}' | '\u{2f98a}' | '\u{2f98b}' | '\u{2f98c}' | '\u{2f98d}'
        | '\u{2f98e}' | '\u{2f98f}' | '\u{2f990}' | '\u{2f991}' | '\u{2f992}' | '\u{2f993}'
        | '\u{2f994}' | '\u{2f995}' | '\u{2f996}' | '\u{2f997}' | '\u{2f998}' | '\u{2f999}'
        | '\u{2f99a}' | '\u{2f99b}' | '\u{2f99c}' | '\u{2f99d}' | '\u{2f99e}' | '\u{2f99f}'
        | '\u{2f9a0}' | '\u{2f9a1}' | '\u{2f9a2}' | '\u{2f9a3}' | '\u{2f9a4}' | '\u{2f9a5}'
        | '\u{2f9a6}' | '\u{2f9a7}' | '\u{2f9a8}' | '\u{2f9a9}' | '\u{2f9aa}' | '\u{2f9ab}'
        | '\u{2f9ac}' | '\u{2f9ad}' | '\u{2f9ae}' | '\u{2f9af}' | '\u{2f9b0}' | '\u{2f9b1}'
        | '\u{2f9b2}' | '\u{2f9b3}' | '\u{2f9b4}' | '\u{2f9b5}' | '\u{2f9b6}' | '\u{2f9b7}'
        | '\u{2f9b8}' | '\u{2f9b9}' | '\u{2f9ba}' | '\u{2f9bb}' | '\u{2f9bc}' | '\u{2f9bd}'
        | '\u{2f9be}' | '\u{2f9bf}' | '\u{2f9c0}' | '\u{2f9c1}' | '\u{2f9c2}' | '\u{2f9c3}'
        | '\u{2f9c4}' | '\u{2f9c5}' | '\u{2f9c6}' | '\u{2f9c7}' | '\u{2f9c8}' | '\u{2f9c9}'
        | '\u{2f9ca}' | '\u{2f9cb}' | '\u{2f9cc}' | '\u{2f9cd}' | '\u{2f9ce}' | '\u{2f9cf}'
        | '\u{2f9d0}' | '\u{2f9d1}' | '\u{2f9d2}' | '\u{2f9d3}' | '\u{2f9d4}' | '\u{2f9d5}'
        | '\u{2f9d6}' | '\u{2f9d7}' | '\u{2f9d8}' | '\u{2f9d9}' | '\u{2f9da}' | '\u{2f9db}'
        | '\u{2f9dc}' | '\u{2f9dd}' | '\u{2f9de}' | '\u{2f9df}' | '\u{2f9e0}' | '\u{2f9e1}'
        | '\u{2f9e2}' | '\u{2f9e3}' | '\u{2f9e4}' | '\u{2f9e5}' | '\u{2f9e6}' | '\u{2f9e7}'
        | '\u{2f9e8}' | '\u{2f9e9}' | '\u{2f9ea}' | '\u{2f9eb}' | '\u{2f9ec}' | '\u{2f9ed}'
        | '\u{2f9ee}' | '\u{2f9ef}' | '\u{2f9f0}' | '\u{2f9f1}' | '\u{2f9f2}' | '\u{2f9f3}'
        | '\u{2f9f4}' | '\u{2f9f5}' | '\u{2f9f6}' | '\u{2f9f7}' | '\u{2f9f8}' | '\u{2f9f9}'
        | '\u{2f9fa}' | '\u{2f9fb}' | '\u{2f9fc}' | '\u{2f9fd}' | '\u{2f9fe}' | '\u{2f9ff}'
        | '\u{2fa00}' | '\u{2fa01}' | '\u{2fa02}' | '\u{2fa03}' | '\u{2fa04}' | '\u{2fa05}'
        | '\u{2fa06}' | '\u{2fa07}' | '\u{2fa08}' | '\u{2fa09}' | '\u{2fa0a}' | '\u{2fa0b}'
        | '\u{2fa0c}' | '\u{2fa0d}' | '\u{2fa0e}' | '\u{2fa0f}' | '\u{2fa10}' | '\u{2fa11}'
        | '\u{2fa12}' | '\u{2fa13}' | '\u{2fa14}' | '\u{2fa15}' | '\u{2fa16}' | '\u{2fa17}'
        | '\u{2fa18}' | '\u{2fa19}' | '\u{2fa1a}' | '\u{2fa1b}' | '\u{2fa1c}' | '\u{2fa1d}' => {
            cjk_compat(c)
        }
        ORC => orc(),
        BOM => bom(),
        _ => Ok(()),
    }
}

/// An invalid Unicode scalar value sequence.
#[derive(Error, Debug)]
pub enum BasicTextError {
    #[error("Color escape sequences are not enabled")]
    ColorEscapeSequence,
    #[error("Control code not valid in text: {0:?}")]
    ControlCode(char),
    #[error("Deprecated Format Characters are deprecated: {0:?}")]
    DeprecatedFormatChar(char),
    #[error("Escape code not valid in text")]
    Escape,
    #[error("Explicit Bidirectional Formatting Characters are unsupported")]
    BidiFormatChar,
    #[error("Interlinear Annotations depend on out-of-band information")]
    Interlinear,
    #[error("Language tagging is a deprecated mechanism")]
    LanguageTag,
    #[error("Line separation is a rich-text function")]
    LineSeparation,
    #[error("Noncharacters are intended for internal use only")]
    NonChar,
    #[error("Paragraph separation is a rich-text function")]
    ParaSeparation,
    #[error("U+FEFF is not necessary in Basic Text")]
    UnneededBOM,
    #[error("U+FFFC depends on out-of-band information")]
    OutOfBand,
    #[error("Unicode discourages use of {0:?}")]
    Discouraged(char),
    #[error("Unrecognized escape sequence")]
    UnrecognizedEscape,
    #[error("Use Standardized Variants instead of CJK Compatibility Ideographs: {0:?}")]
    CJKCompat(char),
    #[error("Use {yes:?} instead of {no:?}")]
    Replacement { yes: Box<[char]>, no: char },
}

#[cold]
fn control(c: char) -> Result<(), BasicTextError> {
    Err(BasicTextError::ControlCode(c))
}

#[cold]
fn replacement(c: char) -> Result<(), BasicTextError> {
    let mut queue = VecDeque::new();
    replace(c, &mut queue);
    Err(BasicTextError::Replacement {
        yes: queue.iter().copied().collect::<Vec<_>>().into_boxed_slice(),
        no: c,
    })
}

#[cold]
fn discouraged(c: char) -> Result<(), BasicTextError> {
    Err(BasicTextError::Discouraged(c))
}

#[cold]
fn deprecated_format_character(c: char) -> Result<(), BasicTextError> {
    Err(BasicTextError::DeprecatedFormatChar(c))
}

#[cold]
fn language_tag() -> Result<(), BasicTextError> {
    Err(BasicTextError::LanguageTag)
}

#[cold]
fn line_separation() -> Result<(), BasicTextError> {
    Err(BasicTextError::LineSeparation)
}

#[cold]
fn para_separation() -> Result<(), BasicTextError> {
    Err(BasicTextError::ParaSeparation)
}

#[cold]
fn bidirectional_formatting_character() -> Result<(), BasicTextError> {
    Err(BasicTextError::BidiFormatChar)
}

#[cold]
fn noncharacter() -> Result<(), BasicTextError> {
    Err(BasicTextError::NonChar)
}

#[cold]
fn cjk_compat(c: char) -> Result<(), BasicTextError> {
    Err(BasicTextError::CJKCompat(c))
}

#[cold]
fn orc() -> Result<(), BasicTextError> {
    Err(BasicTextError::OutOfBand)
}

#[cold]
fn bom() -> Result<(), BasicTextError> {
    Err(BasicTextError::UnneededBOM)
}

#[cold]
fn interlinear_annotation() -> Result<(), BasicTextError> {
    Err(BasicTextError::Interlinear)
}
