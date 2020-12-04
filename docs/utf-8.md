# UTF-8

The *UTF-8* format is simply valid [UTF-8]. On input,
[U+FFFD Substitution of Maximal Subparts] is implicitly applied. On output,
invalid UTF-8 is reported as an error. U+FEFF (BOM) is permitted but not
required.

[UTF-8]: https://www.unicode.org/versions/Unicode13.0.0/ch02.pdf#G11165
[U+FFFD Substitution of Maximal Subparts]: https://www.unicode.org/versions/Unicode13.0.0/ch03.pdf#G66453
