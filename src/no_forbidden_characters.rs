use unicode_normalization::char::canonical_combining_class;

/// An iterator over `char`s which detects occurrences of
/// [Forbidden Characters].
///
/// [Forbidden Characters]: https://unicode.org/reports/tr15/#Forbidding_Characters
pub(crate) struct NoForbiddenCharacters<Inner: Iterator<Item = char>> {
    inner: Inner,
    buffer: Vec<char>,
    pos: usize,
}

impl<Inner: Iterator<Item = char>> NoForbiddenCharacters<Inner> {
    pub(crate) fn new(inner: Inner) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
            pos: 0,
        }
    }
}

impl<Inner: Iterator<Item = char>> Iterator for NoForbiddenCharacters<Inner> {
    type Item = Option<char>;

    #[allow(clippy::match_same_arms)]
    fn next(&mut self) -> Option<Self::Item> {
        if !self.buffer.is_empty() {
            let c = self.buffer[self.pos];
            self.pos += 1;
            if self.pos == self.buffer.len() {
                self.buffer.clear();
            }
            return Some(Some(c));
        }

        self.inner.next().map(|c| match c {
            // http://www.unicode.org/versions/corrigendum3.html
            '\u{f951}' => None,
            // http://www.unicode.org/versions/corrigendum4.html
            '\u{2f868}' | '\u{2f874}' | '\u{2f91f}' | '\u{2f95f}' | '\u{2f9bf}' => None,
            _ => match categorize_c5(c) {
                None => Some(c),
                Some(c5) => {
                    self.buffer.push(c);
                    loop {
                        match self.inner.next() {
                            None => {
                                self.pos = 1;
                                break Some(self.buffer[0]);
                            }
                            Some(c) => match (c, c5) {
                                // Intervening Character(s)
                                (c, _) if canonical_combining_class(c) != 0 => self.buffer.push(c),

                                // Normalized equivalents to [:HangulSyllableType=LV:].
                                ('\u{1161}'..='\u{1175}', _)
                                    if self.buffer.len() == 1
                                        && ('\u{1100}'..='\u{1112}').contains(&self.buffer[0]) =>
                                {
                                    self.buffer.push(c)
                                }

                                // Last Character
                                ('\u{9be}', C5::BangaliVowelSignE)
                                | ('\u{9d7}', C5::BangaliVowelSignE)
                                | ('\u{b3e}', C5::OriyaVowelSignE)
                                | ('\u{b56}', C5::OriyaVowelSignE)
                                | ('\u{b57}', C5::OriyaVowelSignE)
                                | ('\u{bbe}', C5::TamilVowelSignE)
                                | ('\u{bd7}', C5::TamilVowelSignE)
                                | ('\u{bbe}', C5::TamilVowelSignEE)
                                | ('\u{bd7}', C5::TamilLetterO)
                                | ('\u{cc2}', C5::KannadaVowelSignE)
                                | ('\u{cd5}', C5::KannadaVowelSignE)
                                | ('\u{cd6}', C5::KannadaVowelSignE)
                                | ('\u{cd5}', C5::KannadaVowelSignIO)
                                | ('\u{d3e}', C5::MalayalamVowelSignEE)
                                | ('\u{d3e}', C5::MalayalamVowelSignE)
                                | ('\u{d57}', C5::MalayalamVowelSignE)
                                | ('\u{102e}', C5::MyanmarLetterU)
                                | ('\u{dcf}', C5::SinhalaVowelSignKombuva)
                                | ('\u{ddf}', C5::SinhalaVowelSignKombuva)
                                | ('\u{1161}'..='\u{1175}', C5::HangulChoseongKiyeokHieuh)
                                | ('\u{11a8}'..='\u{aac2}', C5::HangulSyllableTypeLV) => {
                                    self.buffer.clear();
                                    break None;
                                }

                                _ => {
                                    self.buffer.push(c);
                                    self.pos = 1;
                                    break Some(self.buffer[0]);
                                }
                            },
                        }
                    }
                }
            },
        })
    }
}

// Table 10. Problem Sequences
#[derive(Clone, Copy)]
enum C5 {
    BangaliVowelSignE,
    OriyaVowelSignE,
    TamilVowelSignE,
    TamilVowelSignEE,
    TamilLetterO,
    KannadaVowelSignE,
    KannadaVowelSignIO,
    MalayalamVowelSignEE,
    MalayalamVowelSignE,
    MyanmarLetterU,
    SinhalaVowelSignKombuva,
    HangulChoseongKiyeokHieuh,
    HangulSyllableTypeLV,
}

fn categorize_c5(c: char) -> Option<C5> {
    Some(match c {
        // https://unicode.org/reports/tr15/#Corrigendum_5_Sequences
        '\u{9c7}' => C5::BangaliVowelSignE,
        '\u{b47}' => C5::OriyaVowelSignE,
        '\u{bc6}' => C5::TamilVowelSignE,
        '\u{bc7}' => C5::TamilVowelSignEE,
        '\u{b92}' => C5::TamilLetterO,
        '\u{cc6}' => C5::KannadaVowelSignE,
        '\u{cbf}' | '\u{cca}' => C5::KannadaVowelSignIO,
        '\u{d47}' => C5::MalayalamVowelSignEE,
        '\u{d46}' => C5::MalayalamVowelSignE,
        '\u{1025}' => C5::MyanmarLetterU,
        '\u{dd9}' => C5::SinhalaVowelSignKombuva,
        '\u{1100}'..='\u{1112}' => C5::HangulChoseongKiyeokHieuh,
        // https://www.unicode.org/Public/13.0.0/ucd/HangulSyllableType.txt
        '\u{ac00}' | '\u{ac1c}' | '\u{ac38}' | '\u{ac54}' | '\u{ac70}' | '\u{ac8c}'
        | '\u{aca8}' | '\u{acc4}' | '\u{ace0}' | '\u{acfc}' | '\u{ad18}' | '\u{ad34}'
        | '\u{ad50}' | '\u{ad6c}' | '\u{ad88}' | '\u{ada4}' | '\u{adc0}' | '\u{addc}'
        | '\u{adf8}' | '\u{ae14}' | '\u{ae30}' | '\u{ae4c}' | '\u{ae68}' | '\u{ae84}'
        | '\u{aea0}' | '\u{aebc}' | '\u{aed8}' | '\u{aef4}' | '\u{af10}' | '\u{af2c}'
        | '\u{af48}' | '\u{af64}' | '\u{af80}' | '\u{af9c}' | '\u{afb8}' | '\u{afd4}'
        | '\u{aff0}' | '\u{b00c}' | '\u{b028}' | '\u{b044}' | '\u{b060}' | '\u{b07c}'
        | '\u{b098}' | '\u{b0b4}' | '\u{b0d0}' | '\u{b0ec}' | '\u{b108}' | '\u{b124}'
        | '\u{b140}' | '\u{b15c}' | '\u{b178}' | '\u{b194}' | '\u{b1b0}' | '\u{b1cc}'
        | '\u{b1e8}' | '\u{b204}' | '\u{b220}' | '\u{b23c}' | '\u{b258}' | '\u{b274}'
        | '\u{b290}' | '\u{b2ac}' | '\u{b2c8}' | '\u{b2e4}' | '\u{b300}' | '\u{b31c}'
        | '\u{b338}' | '\u{b354}' | '\u{b370}' | '\u{b38c}' | '\u{b3a8}' | '\u{b3c4}'
        | '\u{b3e0}' | '\u{b3fc}' | '\u{b418}' | '\u{b434}' | '\u{b450}' | '\u{b46c}'
        | '\u{b488}' | '\u{b4a4}' | '\u{b4c0}' | '\u{b4dc}' | '\u{b4f8}' | '\u{b514}'
        | '\u{b530}' | '\u{b54c}' | '\u{b568}' | '\u{b584}' | '\u{b5a0}' | '\u{b5bc}'
        | '\u{b5d8}' | '\u{b5f4}' | '\u{b610}' | '\u{b62c}' | '\u{b648}' | '\u{b664}'
        | '\u{b680}' | '\u{b69c}' | '\u{b6b8}' | '\u{b6d4}' | '\u{b6f0}' | '\u{b70c}'
        | '\u{b728}' | '\u{b744}' | '\u{b760}' | '\u{b77c}' | '\u{b798}' | '\u{b7b4}'
        | '\u{b7d0}' | '\u{b7ec}' | '\u{b808}' | '\u{b824}' | '\u{b840}' | '\u{b85c}'
        | '\u{b878}' | '\u{b894}' | '\u{b8b0}' | '\u{b8cc}' | '\u{b8e8}' | '\u{b904}'
        | '\u{b920}' | '\u{b93c}' | '\u{b958}' | '\u{b974}' | '\u{b990}' | '\u{b9ac}'
        | '\u{b9c8}' | '\u{b9e4}' | '\u{ba00}' | '\u{ba1c}' | '\u{ba38}' | '\u{ba54}'
        | '\u{ba70}' | '\u{ba8c}' | '\u{baa8}' | '\u{bac4}' | '\u{bae0}' | '\u{bafc}'
        | '\u{bb18}' | '\u{bb34}' | '\u{bb50}' | '\u{bb6c}' | '\u{bb88}' | '\u{bba4}'
        | '\u{bbc0}' | '\u{bbdc}' | '\u{bbf8}' | '\u{bc14}' | '\u{bc30}' | '\u{bc4c}'
        | '\u{bc68}' | '\u{bc84}' | '\u{bca0}' | '\u{bcbc}' | '\u{bcd8}' | '\u{bcf4}'
        | '\u{bd10}' | '\u{bd2c}' | '\u{bd48}' | '\u{bd64}' | '\u{bd80}' | '\u{bd9c}'
        | '\u{bdb8}' | '\u{bdd4}' | '\u{bdf0}' | '\u{be0c}' | '\u{be28}' | '\u{be44}'
        | '\u{be60}' | '\u{be7c}' | '\u{be98}' | '\u{beb4}' | '\u{bed0}' | '\u{beec}'
        | '\u{bf08}' | '\u{bf24}' | '\u{bf40}' | '\u{bf5c}' | '\u{bf78}' | '\u{bf94}'
        | '\u{bfb0}' | '\u{bfcc}' | '\u{bfe8}' | '\u{c004}' | '\u{c020}' | '\u{c03c}'
        | '\u{c058}' | '\u{c074}' | '\u{c090}' | '\u{c0ac}' | '\u{c0c8}' | '\u{c0e4}'
        | '\u{c100}' | '\u{c11c}' | '\u{c138}' | '\u{c154}' | '\u{c170}' | '\u{c18c}'
        | '\u{c1a8}' | '\u{c1c4}' | '\u{c1e0}' | '\u{c1fc}' | '\u{c218}' | '\u{c234}'
        | '\u{c250}' | '\u{c26c}' | '\u{c288}' | '\u{c2a4}' | '\u{c2c0}' | '\u{c2dc}'
        | '\u{c2f8}' | '\u{c314}' | '\u{c330}' | '\u{c34c}' | '\u{c368}' | '\u{c384}'
        | '\u{c3a0}' | '\u{c3bc}' | '\u{c3d8}' | '\u{c3f4}' | '\u{c410}' | '\u{c42c}'
        | '\u{c448}' | '\u{c464}' | '\u{c480}' | '\u{c49c}' | '\u{c4b8}' | '\u{c4d4}'
        | '\u{c4f0}' | '\u{c50c}' | '\u{c528}' | '\u{c544}' | '\u{c560}' | '\u{c57c}'
        | '\u{c598}' | '\u{c5b4}' | '\u{c5d0}' | '\u{c5ec}' | '\u{c608}' | '\u{c624}'
        | '\u{c640}' | '\u{c65c}' | '\u{c678}' | '\u{c694}' | '\u{c6b0}' | '\u{c6cc}'
        | '\u{c6e8}' | '\u{c704}' | '\u{c720}' | '\u{c73c}' | '\u{c758}' | '\u{c774}'
        | '\u{c790}' | '\u{c7ac}' | '\u{c7c8}' | '\u{c7e4}' | '\u{c800}' | '\u{c81c}'
        | '\u{c838}' | '\u{c854}' | '\u{c870}' | '\u{c88c}' | '\u{c8a8}' | '\u{c8c4}'
        | '\u{c8e0}' | '\u{c8fc}' | '\u{c918}' | '\u{c934}' | '\u{c950}' | '\u{c96c}'
        | '\u{c988}' | '\u{c9a4}' | '\u{c9c0}' | '\u{c9dc}' | '\u{c9f8}' | '\u{ca14}'
        | '\u{ca30}' | '\u{ca4c}' | '\u{ca68}' | '\u{ca84}' | '\u{caa0}' | '\u{cabc}'
        | '\u{cad8}' | '\u{caf4}' | '\u{cb10}' | '\u{cb2c}' | '\u{cb48}' | '\u{cb64}'
        | '\u{cb80}' | '\u{cb9c}' | '\u{cbb8}' | '\u{cbd4}' | '\u{cbf0}' | '\u{cc0c}'
        | '\u{cc28}' | '\u{cc44}' | '\u{cc60}' | '\u{cc7c}' | '\u{cc98}' | '\u{ccb4}'
        | '\u{ccd0}' | '\u{ccec}' | '\u{cd08}' | '\u{cd24}' | '\u{cd40}' | '\u{cd5c}'
        | '\u{cd78}' | '\u{cd94}' | '\u{cdb0}' | '\u{cdcc}' | '\u{cde8}' | '\u{ce04}'
        | '\u{ce20}' | '\u{ce3c}' | '\u{ce58}' | '\u{ce74}' | '\u{ce90}' | '\u{ceac}'
        | '\u{cec8}' | '\u{cee4}' | '\u{cf00}' | '\u{cf1c}' | '\u{cf38}' | '\u{cf54}'
        | '\u{cf70}' | '\u{cf8c}' | '\u{cfa8}' | '\u{cfc4}' | '\u{cfe0}' | '\u{cffc}'
        | '\u{d018}' | '\u{d034}' | '\u{d050}' | '\u{d06c}' | '\u{d088}' | '\u{d0a4}'
        | '\u{d0c0}' | '\u{d0dc}' | '\u{d0f8}' | '\u{d114}' | '\u{d130}' | '\u{d14c}'
        | '\u{d168}' | '\u{d184}' | '\u{d1a0}' | '\u{d1bc}' | '\u{d1d8}' | '\u{d1f4}'
        | '\u{d210}' | '\u{d22c}' | '\u{d248}' | '\u{d264}' | '\u{d280}' | '\u{d29c}'
        | '\u{d2b8}' | '\u{d2d4}' | '\u{d2f0}' | '\u{d30c}' | '\u{d328}' | '\u{d344}'
        | '\u{d360}' | '\u{d37c}' | '\u{d398}' | '\u{d3b4}' | '\u{d3d0}' | '\u{d3ec}'
        | '\u{d408}' | '\u{d424}' | '\u{d440}' | '\u{d45c}' | '\u{d478}' | '\u{d494}'
        | '\u{d4b0}' | '\u{d4cc}' | '\u{d4e8}' | '\u{d504}' | '\u{d520}' | '\u{d53c}'
        | '\u{d558}' | '\u{d574}' | '\u{d590}' | '\u{d5ac}' | '\u{d5c8}' | '\u{d5e4}'
        | '\u{d600}' | '\u{d61c}' | '\u{d638}' | '\u{d654}' | '\u{d670}' | '\u{d68c}'
        | '\u{d6a8}' | '\u{d6c4}' | '\u{d6e0}' | '\u{d6fc}' | '\u{d718}' | '\u{d734}'
        | '\u{d750}' | '\u{d76c}' | '\u{d788}' => C5::HangulSyllableTypeLV,
        _ => return None,
    })
}
